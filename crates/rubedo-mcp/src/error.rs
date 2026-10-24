//! Error types and conversions for Model Context Protocol (MCP) communication.

use rubedo_core::reagent::ReagentError;

/// Error variants originating from MCP protocol handling, JSON-RPC serialization, and transports.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Transport channel failure.
    #[error("Transport error: {0}")]
    Transport(String),

    /// JSON-RPC 2.0 protocol error returned by peer.
    #[error("JSON-RPC error [{code}]: {message}")]
    JsonRpc {
        /// Standard JSON-RPC error code.
        code: i64,
        /// Human-readable error description.
        message: String,
    },

    /// Protocol handshake, schema, or message format violation.
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Standard I/O communication failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Tool invocation or execution error.
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
}

impl From<McpError> for ReagentError {
    fn from(err: McpError) -> Self {
        match err {
            McpError::ToolExecution(msg) => ReagentError::Fatal(msg),
            McpError::JsonRpc { code, message } => {
                if code == -32602 {
                    ReagentError::InvalidInput(message)
                } else {
                    ReagentError::Fatal(format!("MCP JSON-RPC [{code}]: {message}"))
                }
            }
            McpError::Serialization(e) => {
                ReagentError::InvalidInput(format!("MCP payload serialization error: {e}"))
            }
            other => ReagentError::Fatal(other.to_string()),
        }
    }
}
