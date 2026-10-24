//! Error hierarchy and conversions for the Alembic WebAssembly sandbox runtime.

use rubedo_core::reagent::ReagentError;

/// Error variants originating from WebAssembly sandbox execution, memory exchange, or isolation violations.
#[derive(Debug, thiserror::Error)]
pub enum AlembicError {
    /// The guest exhausted its allocated instruction fuel quota.
    #[error("Fuel exhausted: execution exceeded instruction limit")]
    FuelExhausted,

    /// Requested memory allocation exceeded the configured maximum boundary.
    #[error("Memory limit exceeded: requested {requested} bytes, max {max} bytes")]
    MemoryLimitExceeded {
        /// Number of bytes requested for allocation or execution.
        requested: usize,
        /// Maximum allowable bytes configured.
        max: usize,
    },

    /// Module compilation, linking, or instantiation failed.
    #[error("Guest instantiation error: {0}")]
    Instantiation(String),

    /// WebAssembly runtime trap occurred during execution.
    #[error("WASM execution trapped: {0}")]
    Trap(String),

    /// Required ABI export was not found in the guest module.
    #[error("Missing exported symbol: {0}")]
    MissingExport(String),

    /// Linear memory access attempt was outside valid guest memory bounds.
    #[error("Memory access out of bounds: pointer {ptr}, length {len}, memory size {size}")]
    OutOfBounds {
        /// Base memory offset.
        ptr: usize,
        /// Buffer length requested.
        len: usize,
        /// Total guest linear memory size.
        size: usize,
    },

    /// JSON serialization or deserialization failure during memory boundary exchange.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Output bytes returned by the guest module were not valid UTF-8.
    #[error("Invalid UTF-8 output from guest: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl From<AlembicError> for ReagentError {
    fn from(err: AlembicError) -> Self {
        match err {
            AlembicError::FuelExhausted => {
                ReagentError::Fatal("WASM fuel exhausted: execution aborted".into())
            }
            AlembicError::MemoryLimitExceeded { requested, max } => {
                ReagentError::Fatal(format!("WASM memory limit exceeded: {requested} > {max}"))
            }
            AlembicError::Trap(msg) => ReagentError::Fatal(format!("WASM trap: {msg}")),
            AlembicError::Instantiation(msg) => {
                ReagentError::Fatal(format!("WASM instantiation failed: {msg}"))
            }
            AlembicError::MissingExport(msg) => {
                ReagentError::Fatal(format!("WASM missing export: {msg}"))
            }
            AlembicError::OutOfBounds { ptr, len, size } => {
                ReagentError::Fatal(format!("WASM memory out of bounds: {ptr}+{len} > {size}"))
            }
            AlembicError::Serialization(e) => {
                ReagentError::InvalidInput(format!("WASM payload serialization error: {e}"))
            }
            AlembicError::Utf8(e) => {
                ReagentError::Fatal(format!("WASM returned non-UTF8 output: {e}"))
            }
        }
    }
}
