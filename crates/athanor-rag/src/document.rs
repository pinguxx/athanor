//! Document and search result data structures for the RAG subsystem.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A text document or passage managed by the Spagyric vector index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for the document or chunk.
    pub id: String,
    /// Textual content of the document.
    pub content: String,
    /// Arbitrary metadata associated with this document (e.g. source, author, timestamps).
    pub metadata: Value,
}

impl Document {
    /// Create a new document with an auto-generated UUID and empty metadata object.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a new document with a specified identifier and empty metadata object.
    pub fn with_id(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a new document with a specified identifier and structured metadata.
    pub fn with_metadata(
        id: impl Into<String>,
        content: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata,
        }
    }
}

/// A ranked search result returned from the vector index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching document or text chunk.
    pub document: Document,
    /// Cosine similarity score between the query and the document (range [-1.0, 1.0], typically [0.0, 1.0]).
    pub score: f32,
}
