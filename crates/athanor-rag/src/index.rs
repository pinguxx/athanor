//! Spagyric vector index managing embeddings and LanceDB storage.

use arrow_array::cast::AsArray;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Connection, DistanceType};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use crate::document::{Document, SearchResult};
use crate::error::RagError;

const DEFAULT_TABLE_NAME: &str = "spagyric_documents";
const EMBEDDING_DIMENSION: usize = 384;

static MODEL_INIT_LOCK: Mutex<()> = Mutex::const_new(());

fn ensure_hf_endpoint() {
    if std::env::var("HF_ENDPOINT")
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        std::env::set_var("HF_ENDPOINT", "https://huggingface.co");
    }
}

fn init_embedding_model() -> Result<TextEmbedding, RagError> {
    ensure_hf_endpoint();
    let mut attempts = 0;
    loop {
        attempts += 1;
        let options = InitOptions {
            model_name: EmbeddingModel::BGESmallENV15,
            show_download_progress: false,
            ..Default::default()
        };
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            TextEmbedding::try_new(options)
        }));
        match res {
            Ok(Ok(model)) => return Ok(model),
            Ok(Err(e)) => {
                if attempts < 3 {
                    std::thread::sleep(std::time::Duration::from_millis(500 * attempts));
                    continue;
                }
                return Err(RagError::Embedding(e.to_string()));
            }
            Err(_panic) => {
                if attempts < 3 {
                    std::thread::sleep(std::time::Duration::from_millis(500 * attempts));
                    continue;
                }
                return Err(RagError::Embedding(
                    "Fastembed panicked during model retrieval".into(),
                ));
            }
        }
    }
}

/// In-process vector index using `fastembed` (BGESmallENV15) and embedded `LanceDB`.
pub struct SpagyricIndex {
    model: Arc<Mutex<TextEmbedding>>,
    db: Connection,
    table_name: String,
    _temp_dir: Option<Arc<TempDir>>,
}

impl SpagyricIndex {
    /// Initialize a new in-memory `SpagyricIndex`.
    ///
    /// Creates an isolated temporary directory for vector storage that is safely cleaned up
    /// when the index is dropped.
    pub async fn new_in_memory() -> Result<Self, RagError> {
        let temp_dir = TempDir::new()?;
        let uri = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| RagError::Config("Invalid UTF-8 in temporary directory path".into()))?;

        let db = lancedb::connect(uri)
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let _guard = MODEL_INIT_LOCK.lock().await;
        let model = init_embedding_model()?;
        drop(_guard);

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            db,
            table_name: DEFAULT_TABLE_NAME.to_string(),
            _temp_dir: Some(Arc::new(temp_dir)),
        })
    }

    /// Initialize a persistent `SpagyricIndex` stored at the given directory path.
    pub async fn new_persistent(path: impl AsRef<Path>) -> Result<Self, RagError> {
        let uri = path
            .as_ref()
            .to_str()
            .ok_or_else(|| RagError::Config("Invalid UTF-8 in database directory path".into()))?;

        let db = lancedb::connect(uri)
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let _guard = MODEL_INIT_LOCK.lock().await;
        let model = init_embedding_model()?;
        drop(_guard);

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            db,
            table_name: DEFAULT_TABLE_NAME.to_string(),
            _temp_dir: None,
        })
    }

    /// Embed and ingest documents into the vector store.
    pub async fn add_documents(&self, docs: &[Document]) -> Result<(), RagError> {
        if docs.is_empty() {
            return Ok(());
        }

        // 1. Generate in-process embeddings
        let texts: Vec<&str> = docs.iter().map(|d| d.content.as_str()).collect();
        let embeddings = {
            let guard = self.model.lock().await;
            guard
                .embed(texts, None)
                .map_err(|e| RagError::Embedding(e.to_string()))?
        };

        // 2. Build Arrow record batch
        let id_array = Arc::new(StringArray::from_iter_values(
            docs.iter().map(|d| d.id.as_str()),
        ));
        let content_array = Arc::new(StringArray::from_iter_values(
            docs.iter().map(|d| d.content.as_str()),
        ));
        let metadata_strings: Vec<String> = docs.iter().map(|d| d.metadata.to_string()).collect();
        let metadata_array = Arc::new(StringArray::from_iter_values(
            metadata_strings.iter().map(|s| s.as_str()),
        ));

        let item_field = Arc::new(Field::new("item", DataType::Float32, true));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                embeddings
                    .into_iter()
                    .map(|v| Some(v.into_iter().map(Some))),
                EMBEDDING_DIMENSION as i32,
            ),
        );

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(item_field, EMBEDDING_DIMENSION as i32),
                false,
            ),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![id_array, content_array, metadata_array, vector_array],
        )?;

        // 3. Persist batch to LanceDB table
        let table_names = self
            .db
            .table_names()
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let batch_reader = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

        if table_names.contains(&self.table_name) {
            let table = self
                .db
                .open_table(&self.table_name)
                .execute()
                .await
                .map_err(|e| RagError::Database(e.to_string()))?;
            table
                .add(Box::new(batch_reader))
                .execute()
                .await
                .map_err(|e| RagError::Database(e.to_string()))?;
        } else {
            self.db
                .create_table(&self.table_name, Box::new(batch_reader))
                .execute()
                .await
                .map_err(|e| RagError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Query the vector store using cosine similarity and return the top-k matching documents.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, RagError> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let table_names = self
            .db
            .table_names()
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        if !table_names.contains(&self.table_name) {
            return Ok(Vec::new());
        }

        // 1. Generate query embedding
        let query_embedding = {
            let guard = self.model.lock().await;
            let mut embeddings = guard
                .embed(vec![trimmed_query], None)
                .map_err(|e| RagError::Embedding(e.to_string()))?;
            embeddings.pop().ok_or_else(|| {
                RagError::Embedding("No embedding produced for search query".into())
            })?
        };

        // 2. Query LanceDB using cosine similarity
        let table = self
            .db
            .open_table(&self.table_name)
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let stream = table
            .query()
            .nearest_to(query_embedding)
            .map_err(|e| RagError::Database(e.to_string()))?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let batches: Vec<RecordBatch> = stream
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| RagError::Database(e.to_string()))?;

        let mut results = Vec::new();

        for batch in batches {
            let id_col = batch
                .column_by_name("id")
                .ok_or_else(|| RagError::Database("Missing 'id' column in query result".into()))?
                .as_string::<i32>();

            let content_col = batch
                .column_by_name("content")
                .ok_or_else(|| {
                    RagError::Database("Missing 'content' column in query result".into())
                })?
                .as_string::<i32>();

            let metadata_col = batch
                .column_by_name("metadata")
                .ok_or_else(|| {
                    RagError::Database("Missing 'metadata' column in query result".into())
                })?
                .as_string::<i32>();

            let distance_col = batch
                .column_by_name("_distance")
                .ok_or_else(|| {
                    RagError::Database("Missing '_distance' column in query result".into())
                })?
                .as_primitive::<Float32Type>();

            for i in 0..batch.num_rows() {
                let distance = distance_col.value(i);
                // LanceDB Cosine distance = 1.0 - Cosine similarity
                let cosine_similarity = 1.0 - distance;
                let meta_str = metadata_col.value(i);
                let metadata: Value = serde_json::from_str(meta_str).unwrap_or(Value::Null);

                results.push(SearchResult {
                    document: Document {
                        id: id_col.value(i).to_string(),
                        content: content_col.value(i).to_string(),
                        metadata,
                    },
                    score: cosine_similarity,
                });
            }
        }

        Ok(results)
    }
}
