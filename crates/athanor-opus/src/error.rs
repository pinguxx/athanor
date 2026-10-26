use crate::vessel::VesselError;
use athanor_crucible::CrucibleError;

/// Top-level error enumeration for Opus workflow operations.
#[derive(Debug, thiserror::Error)]
pub enum OpusError {
    /// Failure during individual Vessel step execution.
    #[error("Vessel execution failed: {0}")]
    Vessel(#[from] VesselError),

    /// Persistence or database failure in Crucible state store.
    #[error("Persistence error: {0}")]
    Crucible(#[from] CrucibleError),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Error in workflow topology definition.
    #[error("Workflow topology error: {0}")]
    Topology(String),

    /// Resumption error during tokenized continuation.
    #[error("Resume error: {0}")]
    Resume(String),
}
