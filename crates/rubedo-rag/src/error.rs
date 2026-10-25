//! Error types for the Spagyric RAG subsystem.

use rubedo_core::reagent::ReagentError;

/// Errors originating from document ingestion, text splitting, embeddings, or vector storage.
#[derive(Debug, thiserror::Error)]
pub enum RagError {
    /// Failure during embedding generation or ONNX model execution.
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Failure interacting with the LanceDB vector database.
    #[error("Database error: {0}")]
    Database(String),

    /// Invalid configuration parameters (e.g. invalid chunk size or overlap).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Document content or metadata error.
    #[error("Document error: {0}")]
    Document(String),

    /// Serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Arrow array or schema error.
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow_schema::ArrowError),

    /// Standard I/O communication failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<RagError> for ReagentError {
    fn from(err: RagError) -> Self {
        match err {
            RagError::Config(msg) | RagError::Document(msg) => ReagentError::InvalidInput(msg),
            RagError::Embedding(msg) => ReagentError::Fatal(format!("Embedding error: {msg}")),
            RagError::Database(msg) => ReagentError::Fatal(format!("Database error: {msg}")),
            RagError::Serialization(e) => {
                ReagentError::InvalidInput(format!("RAG serialization error: {e}"))
            }
            RagError::Arrow(e) => ReagentError::Fatal(format!("Arrow error: {e}")),
            RagError::Io(e) => ReagentError::Fatal(format!("I/O error: {e}")),
        }
    }
}
