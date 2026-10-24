# Phase 5: Spagyric RAG & In-Process Vector Store

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-rag`
- **Primary Goal**: Construct an embedded, zero-infrastructure Retrieval-Augmented Generation (RAG) subsystem. Utilizes `fastembed-rs` (model `BGESmallENV15`) for in-process ONNX embeddings and embedded `LanceDB` for high-performance vector search within a sub-40MB memory footprint.

---

## 2. Detailed Task Breakdown

### Task 5.1: Scaffold `rubedo-rag`
- Create `crates/rubedo-rag/Cargo.toml` with dependencies:
  - `fastembed = "3.8"`
  - `lancedb = "0.10"`
  - `rubedo-core`, `serde`, `serde_json`, `tokio`, `thiserror`, `tracing`

### Task 5.2: Implement Document Models & Text Splitters
- In `crates/rubedo-rag/src/document.rs`:
  ```rust
  #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
  pub struct Document {
      pub id: String,
      pub content: String,
      pub metadata: serde_json::Value,
  }

  #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
  pub struct SearchResult {
      pub document: Document,
      pub score: f32,
  }
  ```
- In `crates/rubedo-rag/src/splitter.rs`:
  - Implement `RecursiveCharacterSplitter`:
    - Splits on structural delimiters (`\n\n`, `\n`, `. `, ` `).
    - Configurable `chunk_size` (e.g. 500 characters) and `chunk_overlap` (e.g. 50 characters).

### Task 5.3: Implement `SpagyricIndex`
- In `crates/rubedo-rag/src/index.rs`:
  - Embeddings: lazy initialization of `TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallENV15))`.
  - LanceDB connection: in-memory or embedded temp directory.
  - Implement methods:
    ```rust
    pub struct SpagyricIndex { /* fastembed + lancedb handle */ }

    impl SpagyricIndex {
        pub async fn new_in_memory() -> Result<Self, RagError>;
        pub async fn add_documents(&self, docs: &[Document]) -> Result<(), RagError>;
        pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, RagError>;
    }
    ```

### Task 5.4: Implement Search Reagent for Artificers
- In `crates/rubedo-rag/src/reagent.rs`:
  - Implement `RagSearchReagent`:
    - Wraps `Arc<SpagyricIndex>`.
    - Implements `rubedo_core::reagent::Reagent`.
    - Exposes schema `{ "query": "string", "limit": "integer?" }`.
    - Returns JSON array of top matching passages with similarity scores.

---

## 3. Verification & Acceptance Criteria

### Integration Test
Create `crates/rubedo-rag/tests/semantic_search_test.rs`:
1. Ingest a multi-paragraph technical passage:
   - Paragraph A: Details regarding solar energy conversion and photovoltaic cells.
   - Paragraph B: Details regarding cryogenic propellant combustion in rocket engines.
2. Query with a semantic synonym not explicitly in the text:
   - Query: `"spacecraft liquid fuel reaction"`
3. Verify:
   - Top result corresponds to Paragraph B (rocket engines).
   - Cosine similarity score $> 0.70$.
   - Embedding and search executed completely in-process.

### Execution Command
```bash
cargo test -p rubedo-rag
```
