//! # Athanor MCP Gateway
//!
//! Bidirectional Model Context Protocol (MCP 2024-11-05+) bridge for Athanor agent systems.
//!
//! This crate enables Athanor agents to:
//! 1. Expose native `#[reagent]` tools to external clients (e.g. Claude Desktop, Cursor) via [`McpServer`].
//! 2. Discover and wrap external MCP tools dynamically into native [`athanor_core::reagent::Reagent`] implementations via [`McpClient`].
//! 3. Communicate across in-memory duplex channels ([`ChannelTransport`]) or standard I/O ([`StdioTransport`]).

#![deny(missing_docs)]

pub mod client;
pub mod error;
pub mod protocol;
pub mod server;
pub mod transport;

pub use client::{DynamicMcpReagent, McpClient};
pub use error::McpError;
pub use protocol::{
    CallToolParams, CallToolResult, InitializeResult, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, ListToolsResult, RequestId, ToolDefinition, PROTOCOL_VERSION,
};
pub use server::McpServer;
pub use transport::{ChannelTransport, StdioTransport, Transport};

/// Commonly used types and traits for convenient import.
pub mod prelude {
    pub use crate::client::{DynamicMcpReagent, McpClient};
    pub use crate::error::McpError;
    pub use crate::protocol::{CallToolResult, ToolDefinition, PROTOCOL_VERSION};
    pub use crate::server::McpServer;
    pub use crate::transport::{ChannelTransport, StdioTransport, Transport};
}
