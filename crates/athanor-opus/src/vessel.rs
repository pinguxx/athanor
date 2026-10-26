use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::OpusContext;

/// Execution status outcome produced by a Vessel step.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VesselStatus {
    /// Step is scheduled and awaiting execution.
    Pending,
    /// Step is actively executing.
    Active,
    /// Step execution is suspended awaiting external signal or human intervention.
    Suspended {
        /// Cryptographically unique resume token required to resume workflow.
        resume_token: String,
    },
    /// Step successfully completed and transmuted state with optional output.
    Transmuted {
        /// Output value produced by this Vessel step.
        output: Value,
    },
    /// Step encountered a non-recoverable error.
    Corrupted {
        /// Diagnostic error description.
        error: String,
    },
}

/// Errors originating from Vessel execution.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum VesselError {
    /// Step execution logic failed.
    #[error("Vessel execution failed: {0}")]
    Execution(String),

    /// Required input state missing or invalid.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Fatal unrecoverable failure.
    #[error("Fatal vessel failure: {0}")]
    Fatal(String),
}

/// Individual execution node contract within an Opus workflow DAG.
#[async_trait]
pub trait Vessel: Send + Sync + 'static {
    /// Unique identifier for this step within the workflow DAG.
    fn id(&self) -> &'static str;

    /// Execute step logic, mutating context state or suspending execution.
    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError>;
}
