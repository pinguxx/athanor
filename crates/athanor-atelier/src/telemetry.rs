//! Telemetry pipeline, event definitions, and SSE broadcaster.

use serde::{Deserialize, Serialize};

use crate::error::AtelierError;

/// Individual observability and execution events emitted across the Athanor framework.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum AtelierEvent {
    /// A workflow Vessel node began execution.
    VesselStarted {
        /// Associated workflow run identifier.
        run_id: String,
        /// Vessel node identifier.
        vessel_id: String,
        /// Unix epoch timestamp in milliseconds.
        timestamp: u64,
    },
    /// A workflow Vessel node finished execution.
    VesselCompleted {
        /// Associated workflow run identifier.
        run_id: String,
        /// Vessel node identifier.
        vessel_id: String,
        /// Transition status ('Transmuted', 'Failed', etc.).
        status: String,
        /// Unix epoch timestamp in milliseconds.
        timestamp: u64,
    },
    /// A workflow Vessel node suspended execution awaiting external resumption.
    VesselSuspended {
        /// Associated workflow run identifier.
        run_id: String,
        /// Vessel node identifier.
        vessel_id: String,
        /// Required resume token.
        resume_token: String,
        /// Unix epoch timestamp in milliseconds.
        timestamp: u64,
    },
    /// Artificer LLM token consumption metrics.
    TokenUsage {
        /// Persona or identifier of the Artificer.
        artificer: String,
        /// Number of prompt tokens ingested.
        prompt_tokens: u32,
        /// Number of completion tokens generated.
        completion_tokens: u32,
        /// Aggregate total tokens consumed.
        total_tokens: u32,
        /// Unix epoch timestamp in milliseconds.
        timestamp: u64,
    },
    /// Standard structured log trace.
    LogMessage {
        /// Severity level ('INFO', 'WARN', 'ERROR', 'DEBUG', 'TRACE').
        level: String,
        /// Module target or component path.
        target: String,
        /// Formatted log payload.
        message: String,
        /// Unix epoch timestamp in milliseconds.
        timestamp: u64,
    },
}

impl AtelierEvent {
    /// Construct a `VesselStarted` event with the current timestamp.
    pub fn vessel_started(run_id: impl Into<String>, vessel_id: impl Into<String>) -> Self {
        Self::VesselStarted {
            run_id: run_id.into(),
            vessel_id: vessel_id.into(),
            timestamp: current_epoch_ms(),
        }
    }

    /// Construct a `VesselCompleted` event with the current timestamp.
    pub fn vessel_completed(
        run_id: impl Into<String>,
        vessel_id: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self::VesselCompleted {
            run_id: run_id.into(),
            vessel_id: vessel_id.into(),
            status: status.into(),
            timestamp: current_epoch_ms(),
        }
    }

    /// Construct a `VesselSuspended` event with the current timestamp.
    pub fn vessel_suspended(
        run_id: impl Into<String>,
        vessel_id: impl Into<String>,
        resume_token: impl Into<String>,
    ) -> Self {
        Self::VesselSuspended {
            run_id: run_id.into(),
            vessel_id: vessel_id.into(),
            resume_token: resume_token.into(),
            timestamp: current_epoch_ms(),
        }
    }

    /// Construct a `TokenUsage` event with calculated total and current timestamp.
    pub fn token_usage(
        artificer: impl Into<String>,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Self {
        Self::TokenUsage {
            artificer: artificer.into(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
            timestamp: current_epoch_ms(),
        }
    }

    /// Construct a `LogMessage` event with the current timestamp.
    pub fn log_message(
        level: impl Into<String>,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::LogMessage {
            level: level.into(),
            target: target.into(),
            message: message.into(),
            timestamp: current_epoch_ms(),
        }
    }
}

/// Broadcaster streaming execution events to connected SSE clients.
#[derive(Clone)]
pub struct TelemetryBroadcaster {
    sender: tokio::sync::broadcast::Sender<AtelierEvent>,
}

impl TelemetryBroadcaster {
    /// Create a new broadcaster with the given queue buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity.max(16));
        Self { sender }
    }

    /// Broadcast an execution event to all active subscribers.
    pub fn broadcast(&self, event: AtelierEvent) -> Result<usize, AtelierError> {
        self.sender
            .send(event)
            .map_err(|e| AtelierError::Telemetry(format!("Channel broadcast failed: {e}")))
    }

    /// Subscribe a new receiver to the broadcast stream.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AtelierEvent> {
        self.sender.subscribe()
    }

    /// Return the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Create a tracing subscriber Layer hooked into this broadcaster.
    pub fn tracing_layer(&self) -> AtelierTracingLayer {
        AtelierTracingLayer {
            sender: self.sender.clone(),
        }
    }
}

impl Default for TelemetryBroadcaster {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Custom `tracing_subscriber::Layer` intercepting log events and relaying them to Atelier.
#[derive(Clone)]
pub struct AtelierTracingLayer {
    sender: tokio::sync::broadcast::Sender<AtelierEvent>,
}

impl<S> tracing_subscriber::Layer<S> for AtelierTracingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let level = meta.level().to_string();
        let target = meta.target().to_string();

        let mut visitor = EventMessageVisitor::default();
        event.record(&mut visitor);

        let message = match visitor.message {
            Some(msg) => msg,
            None => meta.name().to_string(),
        };

        let log_event = AtelierEvent::LogMessage {
            level,
            target,
            message,
            timestamp: current_epoch_ms(),
        };

        let _ = self.sender.send(log_event);
    }
}

#[derive(Default)]
struct EventMessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for EventMessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}").trim_matches('"').to_string());
        } else if self.message.is_none() {
            self.message = Some(format!("{}: {:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else if self.message.is_none() {
            self.message = Some(format!("{}: {}", field.name(), value));
        }
    }
}

fn current_epoch_ms() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}
