/// Errors originating from Crucible state store operations.
#[derive(Debug, thiserror::Error)]
pub enum CrucibleError {
    /// SQLite execution failure.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Specified run ID was not found.
    #[error("Run not found: {0}")]
    RunNotFound(String),

    /// Specified resume token was not found or has already been consumed.
    #[error("Suspended token not found or already consumed: {0}")]
    TokenNotFound(String),

    /// Database migration failure.
    #[error("Migration error: {0}")]
    Migration(String),

    /// State consistency invariant violated.
    #[error("Invariant error: {0}")]
    Invariant(String),
}
