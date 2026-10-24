//! JSON-RPC 2.0 and Model Context Protocol (MCP 2024-11-05+) message schemas.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard Model Context Protocol version string supported by Rubedo.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Standard JSON-RPC 2.0 error codes.
pub mod error_codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i64 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i64 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i64 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i64 = -32603;
}

/// JSON-RPC 2.0 request or notification identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Numerical request identifier.
    Number(i64),
    /// String request identifier.
    String(String),
}

impl From<i64> for RequestId {
    fn from(val: i64) -> Self {
        Self::Number(val)
    }
}

impl From<String> for RequestId {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<&str> for RequestId {
    fn from(val: &str) -> Self {
        Self::String(val.to_string())
    }
}

/// A JSON-RPC 2.0 request or notification message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol specification version (must be "2.0").
    pub jsonrpc: String,
    /// Optional identifier. If `None`, the message is a notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Name of the remote procedure to invoke.
    pub method: String,
    /// Parameters associated with the invocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Creates a new JSON-RPC 2.0 request with an identifier.
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id.into()),
            method: method.into(),
            params,
        }
    }

    /// Creates a new JSON-RPC 2.0 notification (no identifier, no response expected).
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.into(),
            params,
        }
    }

    /// Returns `true` if this message is a notification.
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// A JSON-RPC 2.0 response message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol specification version (must be "2.0").
    pub jsonrpc: String,
    /// Correlated request identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Result payload if invocation succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error object if invocation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Construct a successful response.
    pub fn success(id: Option<RequestId>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Construct an error response.
    pub fn error(id: Option<RequestId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 structured error model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Standard or application-specific error code.
    pub code: i64,
    /// Short summary of the error.
    pub message: String,
    /// Optional additional diagnostic data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Construct a new error with an arbitrary code and message.
    pub fn new(code: i64, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    /// Construct a `-32700` Parse Error.
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::new(error_codes::PARSE_ERROR, msg, None)
    }

    /// Construct a `-32600` Invalid Request error.
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_REQUEST, msg, None)
    }

    /// Construct a `-32601` Method Not Found error.
    pub fn method_not_found(msg: impl Into<String>) -> Self {
        Self::new(error_codes::METHOD_NOT_FOUND, msg, None)
    }

    /// Construct a `-32602` Invalid Params error.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_PARAMS, msg, None)
    }

    /// Construct a `-32603` Internal Error.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, msg, None)
    }
}

/// Information describing the client communicating via MCP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client implementation name.
    pub name: String,
    /// Client implementation version.
    pub version: String,
}

/// Information describing the MCP server implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server implementation name.
    pub name: String,
    /// Server implementation version.
    pub version: String,
}

/// Client capability declarations.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Tool support capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
}

/// Server tool-specific capability declarations.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether the server emits notifications when the tools list changes.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Server capability declarations.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tool support capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Handshake parameters sent by client during `initialize`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version requested by the client.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client declared capabilities.
    pub capabilities: ClientCapabilities,
    /// Information identifying the client.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Handshake result returned by server in response to `initialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Negotiated protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Information identifying the server.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Schema definition for an MCP tool discovered via `tools/list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique name of the tool.
    pub name: String,
    /// Optional human-readable description of what the tool accomplishes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema describing expected tool arguments.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result returned in response to a `tools/list` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Discovered list of tool definitions.
    pub tools: Vec<ToolDefinition>,
    /// Optional pagination cursor for additional tools.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/// Parameters for a `tools/call` invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Name of the tool to execute.
    pub name: String,
    /// Invocation arguments matching the tool's input schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Text content block in a tool invocation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextContent {
    /// Type discriminator (must be "text").
    #[serde(rename = "type")]
    pub r#type: String,
    /// Textual payload.
    pub text: String,
}

impl TextContent {
    /// Creates a new text content item.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            r#type: "text".to_string(),
            text: text.into(),
        }
    }
}

/// Result of a `tools/call` execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content segments generated by the tool.
    pub content: Vec<TextContent>,
    /// Whether tool execution concluded in an error.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "isError")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    /// Construct a successful tool result with single text payload.
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![TextContent::text(text)],
            is_error: Some(false),
        }
    }

    /// Construct an error tool result with explanatory diagnostic text.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![TextContent::text(message)],
            is_error: Some(true),
        }
    }
}
