use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Zero-lifetime owned value envelope guaranteeing thread-safe, lifetime-free transfers.
///
/// Wraps payloads in `'static + Send + Sync` containers with unique tracking identifiers,
/// timestamps, and optional distributed correlation IDs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envelope<T> {
    /// Unique identifier for this envelope.
    pub id: Uuid,
    /// Unix timestamp in milliseconds when the envelope was minted.
    pub timestamp_ms: u64,
    /// Optional correlation identifier for distributed tracing.
    pub correlation_id: Option<String>,
    /// The inner owned payload.
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Wrap a payload into a freshly minted `Envelope`.
    pub fn new(payload: T) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            id: Uuid::new_v4(),
            timestamp_ms,
            correlation_id: None,
            payload,
        }
    }

    /// Set an explicit correlation ID for this envelope.
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Reference the inner payload.
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Mutably reference the inner payload.
    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Consume the envelope and extract the inner owned payload.
    pub fn into_inner(self) -> T {
        self.payload
    }

    /// Transform the inner payload while preserving envelope metadata.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Envelope<U> {
        Envelope {
            id: self.id,
            timestamp_ms: self.timestamp_ms,
            correlation_id: self.correlation_id,
            payload: f(self.payload),
        }
    }
}
