use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Error model representing failures during Reagent input validation or execution.
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReagentError {
    /// Input parameters failed schema validation or deserialization.
    #[error("Validation failed: {0}")]
    InvalidInput(String),

    /// A recoverable failure occurred where the LLM can self-correct given guidance.
    #[error("Recoverable error: {message}. Guidance: {guidance}")]
    Recoverable {
        /// Diagnostic message explaining what went wrong.
        message: String,
        /// Concrete instruction informing the agent how to correct the input.
        guidance: String,
        /// Optional structured suggestion for parameter correction.
        suggested_fix: Option<Value>,
    },

    /// An unrecoverable, fatal failure occurred during reagent execution.
    #[error("Fatal reagent failure: {0}")]
    Fatal(String),
}

impl ReagentError {
    /// Construct an `InvalidInput` error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Construct a `Recoverable` error with diagnostic message and corrective guidance.
    pub fn recoverable(message: impl Into<String>, guidance: impl Into<String>) -> Self {
        Self::Recoverable {
            message: message.into(),
            guidance: guidance.into(),
            suggested_fix: None,
        }
    }

    /// Construct a `Recoverable` error with message, guidance, and a suggested payload fix.
    pub fn recoverable_with_fix(
        message: impl Into<String>,
        guidance: impl Into<String>,
        suggested_fix: Value,
    ) -> Self {
        Self::Recoverable {
            message: message.into(),
            guidance: guidance.into(),
            suggested_fix: Some(suggested_fix),
        }
    }

    /// Construct a `Fatal` error.
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::Fatal(message.into())
    }

    /// Returns `true` if this error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Recoverable { .. })
    }

    /// Returns `true` if this error represents invalid input validation.
    pub fn is_invalid_input(&self) -> bool {
        matches!(self, Self::InvalidInput(_))
    }

    /// Returns `true` if this error is a fatal failure.
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }
}

impl From<serde_json::Error> for ReagentError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<String> for ReagentError {
    fn from(err: String) -> Self {
        Self::Fatal(err)
    }
}

impl From<&str> for ReagentError {
    fn from(err: &str) -> Self {
        Self::Fatal(err.to_string())
    }
}

/// The fundamental alchemical capability trait for executable agent tools.
///
/// Enforces `'static + Send + Sync` guarantees to prevent lifetime annotations
/// and facilitate autonomous agent tool generation.
#[async_trait]
pub trait Reagent: Send + Sync + 'static {
    /// Canonical unique identifier for the reagent.
    fn name(&self) -> &'static str;

    /// Human- and LLM-readable description of what this capability achieves.
    fn description(&self) -> &'static str;

    /// JSON Schema describing the expected structure of input arguments.
    fn input_schema(&self) -> Value;

    /// JSON Schema describing the structure of the returned payload.
    fn output_schema(&self) -> Value;

    /// Execute the reagent capability with validated JSON arguments.
    async fn transmute(&self, input: Value) -> Result<Value, ReagentError>;
}
