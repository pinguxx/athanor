use crate::reagent::ReagentError;

/// Top-level error enumeration for core Rubedo operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum RubedoError {
    /// Errors originating from reagent execution or schema validation.
    #[error("Reagent error: {0}")]
    Reagent(#[from] ReagentError),

    /// Errors in context storage or state retrieval.
    #[error("Context error: {0}")]
    Context(String),

    /// Errors in Artificer creation or configuration.
    #[error("Artificer configuration error: {0}")]
    Artificer(String),

    /// General configuration or serialization error.
    #[error("Configuration error: {0}")]
    Config(String),
}
