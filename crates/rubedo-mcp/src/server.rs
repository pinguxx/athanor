//! Model Context Protocol (MCP 2024-11-05+) server implementation.

use std::collections::HashMap;
use std::sync::Arc;
use rubedo_core::reagent::Reagent;
use tokio::sync::RwLock;

use crate::error::McpError;
use crate::protocol::{
    CallToolParams, CallToolResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    ListToolsResult, ServerCapabilities, ServerInfo, ToolDefinition, ToolsCapability,
    PROTOCOL_VERSION,
};
use crate::transport::Transport;

/// MCP server exposing native `#[reagent]` capabilities over any bidirectional transport.
pub struct McpServer {
    name: String,
    version: String,
    reagents: Arc<RwLock<HashMap<String, Arc<dyn Reagent>>>>,
}

impl McpServer {
    /// Create a new `McpServer` with the specified name and version.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            reagents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a native [`Reagent`] tool to be exposed to external MCP clients.
    pub async fn register_reagent(&self, reagent: Arc<dyn Reagent>) {
        let mut map = self.reagents.write().await;
        map.insert(reagent.name().to_string(), reagent);
    }

    /// Process an incoming raw JSON-RPC string message and produce an optional response string.
    ///
    /// Returns `Ok(Some(response_json))` when a response is required, or `Ok(None)` for
    /// notifications that do not expect a reply.
    pub async fn handle_message(&self, raw: &str) -> Result<Option<String>, McpError> {
        let request: JsonRpcRequest = match serde_json::from_str(raw) {
            Ok(req) => req,
            Err(e) => {
                let err_resp = JsonRpcResponse::error(
                    None,
                    JsonRpcError::parse_error(format!("Failed to parse JSON-RPC payload: {e}")),
                );
                return Ok(Some(serde_json::to_string(&err_resp)?));
            }
        };

        // Notifications (id == None) do not expect a response
        if request.is_notification() {
            tracing::trace!(method = %request.method, "Received MCP notification");
            return Ok(None);
        }

        let response = match request.method.as_str() {
            "initialize" => {
                let server_info = ServerInfo {
                    name: self.name.clone(),
                    version: self.version.clone(),
                };
                let capabilities = ServerCapabilities {
                    tools: Some(ToolsCapability {
                        list_changed: Some(false),
                    }),
                };
                let result = serde_json::json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": capabilities,
                    "serverInfo": server_info,
                });
                JsonRpcResponse::success(request.id, result)
            }

            "tools/list" => {
                let map = self.reagents.read().await;
                let tools: Vec<ToolDefinition> = map
                    .values()
                    .map(|r| ToolDefinition {
                        name: r.name().to_string(),
                        description: Some(r.description().to_string()),
                        input_schema: r.input_schema(),
                    })
                    .collect();

                let result = ListToolsResult {
                    tools,
                    next_cursor: None,
                };
                let result_value = serde_json::to_value(result)?;
                JsonRpcResponse::success(request.id, result_value)
            }

            "tools/call" => {
                let params: CallToolParams = match request.params {
                    Some(p) => match serde_json::from_value(p) {
                        Ok(call_params) => call_params,
                        Err(e) => {
                            let resp = JsonRpcResponse::error(
                                request.id,
                                JsonRpcError::invalid_params(format!("Invalid tools/call parameters: {e}")),
                            );
                            return Ok(Some(serde_json::to_string(&resp)?));
                        }
                    },
                    None => {
                        let resp = JsonRpcResponse::error(
                            request.id,
                            JsonRpcError::invalid_params("Missing parameters for tools/call"),
                        );
                        return Ok(Some(serde_json::to_string(&resp)?));
                    }
                };

                let reagent_opt = {
                    let map = self.reagents.read().await;
                    map.get(&params.name).cloned()
                };

                match reagent_opt {
                    Some(reagent) => {
                        let args = params.arguments.unwrap_or_else(|| serde_json::json!({}));
                        match reagent.transmute(args).await {
                            Ok(output) => {
                                let text = match serde_json::to_string(&output) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        let err_tool = CallToolResult::error(format!(
                                            "Failed to serialize tool output: {e}"
                                        ));
                                        let err_val = serde_json::to_value(err_tool)?;
                                        return Ok(Some(serde_json::to_string(&JsonRpcResponse::success(
                                            request.id,
                                            err_val,
                                        ))?));
                                    }
                                };
                                let tool_result = CallToolResult::success(text);
                                let val = serde_json::to_value(tool_result)?;
                                JsonRpcResponse::success(request.id, val)
                            }
                            Err(err) => {
                                let tool_result = CallToolResult::error(err.to_string());
                                let val = serde_json::to_value(tool_result)?;
                                JsonRpcResponse::success(request.id, val)
                            }
                        }
                    }
                    None => JsonRpcResponse::error(
                        request.id,
                        JsonRpcError::method_not_found(format!("Tool '{}' not found", params.name)),
                    ),
                }
            }

            "ping" => JsonRpcResponse::success(request.id, serde_json::json!({})),

            unknown_method => JsonRpcResponse::error(
                request.id,
                JsonRpcError::method_not_found(format!("Method '{unknown_method}' not recognized")),
            ),
        };

        let response_str = serde_json::to_string(&response)?;
        Ok(Some(response_str))
    }

    /// Execute the server loop reading from and replying into the given transport.
    ///
    /// The loop terminates gracefully when the transport yields `None` (EOF / closed).
    pub async fn run(&self, transport: Arc<dyn Transport>) -> Result<(), McpError> {
        loop {
            match transport.receive().await? {
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    if let Some(response) = self.handle_message(trimmed).await? {
                        transport.send(response).await?;
                    }
                }
                None => {
                    tracing::debug!("MCP transport closed; server loop exiting gracefully");
                    break;
                }
            }
        }

        Ok(())
    }
}
