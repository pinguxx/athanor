use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Persistent record of an Opus workflow execution run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpusRunRecord {
    /// Unique identifier for this workflow run.
    pub run_id: String,
    /// Name of the Opus workflow template.
    pub opus_name: String,
    /// Lifecycle status ('Pending', 'Active', 'Suspended', 'Completed', 'Failed').
    pub status: String,
    /// JSON snapshot of shared workflow memory state.
    pub current_state: Value,
    /// Creation timestamp string.
    pub created_at: String,
    /// Last update timestamp string.
    pub updated_at: String,
}

/// Journal record of an individual Vessel execution transition event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VesselEventRecord {
    /// Monotonically increasing event sequence identifier.
    pub event_id: i64,
    /// Associated workflow run identifier.
    pub run_id: String,
    /// Identifier of the Vessel node that produced the event.
    pub vessel_id: String,
    /// Event transition status ('Active', 'Transmuted', 'Suspended', 'Corrupted').
    pub status: String,
    /// Optional structured output or diagnostic error payload.
    pub payload: Option<Value>,
    /// Timestamp when event was committed.
    pub recorded_at: String,
}

/// Persistent record of an active, unconsumed resumption token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuspendedTokenRecord {
    /// Unforgeable resumption token string (UUIDv4).
    pub token: String,
    /// Associated workflow run identifier.
    pub run_id: String,
    /// Target Vessel node awaiting resumption.
    pub vessel_id: String,
    /// Mint timestamp string.
    pub created_at: String,
}
