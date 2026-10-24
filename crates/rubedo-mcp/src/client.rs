//! Model Context Protocol (MCP 2024-11-05+) client implementation and dynamic reagent wrapping.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use async_trait::async_trait;
use rubedo_core::reagent::{Reagent, ReagentError};
use serde_json::Value;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;

use crate::error::McpError;
use crate::protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcRequest, JsonRpcResponse, ListToolsResult, RequestId,
    ToolDefinition, PROTOCOL_VERSION,
};
use crate::transport::Transport;

type ResponseSender = oneshot::Sender<Result<JsonRpcResponse, McpError>>;
type PendingMap = Arc<Mutex<HashMap<i64, ResponseSender>>>;

/// MCP client connecting to an external MCP server to discover and invoke remote tools.
pub struct McpClient {
    transport: Arc<dyn Transport>,
    next_id: AtomicI64,
    pending: PendingMap,
    _receiver_task: JoinHandle<()>,
}

impl McpClient {
    /// Create and initialize an `McpClient` managing duplex communication over the provided transport.
    ///
    /// Spawns a background listener task to correlate incoming JSON-RPC responses with requests.
    pub fn new(transport: Arc<dyn Transport>) -> Arc<Self> {
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = Arc::clone(&pending);
        let transport_clone = Arc::clone(&transport);


        let receiver_task = tokio::spawn(async move {
            loop {
                match transport_clone.receive().await {
                    Ok(Some(line)) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<JsonRpcResponse>(trimmed) {
                            Ok(resp) => {
                                if let Some(RequestId::Number(id)) = resp.id {
                                    let mut map = pending_clone.lock().await;
                                    if let Some(tx) = map.remove(&id) {
                                        let _ = tx.send(Ok(resp));
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    raw = %trimmed,
                                    "Failed to parse incoming response JSON-RPC"
                                );
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::debug!("Transport closed; client receiver task terminating");
                        let mut map = pending_clone.lock().await;
                        for (_, tx) in map.drain() {
                            let _ = tx.send(Err(McpError::Transport(
                                "Transport connection closed".to_string(),
                            )));
                        }
                        break;
                    }
                    Err(err) => {
                        tracing::error!(
                            error = %err,
                            "Transport error; client receiver task terminating"
                        );
                        let mut map = pending_clone.lock().await;
                        for (_, tx) in map.drain() {
                            let _ = tx.send(Err(McpError::Transport(err.to_string())));
                        }
                        break;
                    }
                }
            }
        });

        Arc::new(Self {
            transport,
            next_id: AtomicI64::new(1),
            pending,
            _receiver_task: receiver_task,
        })
    }

    /// Transmit a JSON-RPC request and await the correlated response.
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value, McpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut map = self.pending.lock().await;
            map.insert(id, tx);
        }

        let rpc_request = JsonRpcRequest::new(RequestId::Number(id), method, params);
        let msg = serde_json::to_string(&rpc_request)?;

        if let Err(e) = self.transport.send(msg).await {
            let mut map = self.pending.lock().await;
            map.remove(&id);
            return Err(e);
        }

        let response = match rx.await {
            Ok(res) => res?,
            Err(_) => {
                return Err(McpError::Transport(
                    "Response receiver channel dropped unexpectedly".to_string(),
                ));
            }
        };

        if let Some(err) = response.error {
            return Err(McpError::JsonRpc {
                code: err.code,
                message: err.message,
            });
        }

        response
            .result
            .ok_or_else(|| McpError::Protocol("Received response without result or error".into()))
    }

    /// Send a JSON-RPC notification message without awaiting a reply.
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<(), McpError> {
        let rpc_notif = JsonRpcRequest::notification(method, params);
        let msg = serde_json::to_string(&rpc_notif)?;
        self.transport.send(msg).await
    }

    /// Perform the standard MCP `initialize` handshake with the remote server.
    pub async fn initialize(
        &self,
        client_name: &str,
        client_version: &str,
    ) -> Result<InitializeResult, McpError> {
        let params = InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                tools: Some(serde_json::json!({})),
            },
            client_info: ClientInfo {
                name: client_name.to_string(),
                version: client_version.to_string(),
            },
        };

        let result_val = self
            .request("initialize", Some(serde_json::to_value(params)?))
            .await?;

        let init_result: InitializeResult = serde_json::from_value(result_val)?;

        // Send notifications/initialized confirming successful handshake
        self.notify("notifications/initialized", None).await?;

        Ok(init_result)
    }

    /// Fetch all remote tool definitions using the `tools/list` RPC method.
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>, McpError> {
        let result_val = self.request("tools/list", Some(serde_json::json!({}))).await?;
        let list_result: ListToolsResult = serde_json::from_value(result_val)?;
        Ok(list_result.tools)
    }

    /// Execute a remote tool via `tools/call` and parse its return value.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments: Some(arguments),
        };

        let result_val = self
            .request("tools/call", Some(serde_json::to_value(params)?))
            .await?;

        // Attempt to parse standard CallToolResult
        if let Ok(tool_res) = serde_json::from_value::<CallToolResult>(result_val.clone()) {
            if tool_res.is_error.unwrap_or(false) {
                let err_msg = tool_res
                    .content
                    .into_iter()
                    .map(|c| c.text)
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(McpError::ToolExecution(err_msg));
            }

            if let Some(first) = tool_res.content.into_iter().next() {
                // If text is valid JSON, parse it into structured Value; otherwise return string Value
                if let Ok(parsed_json) = serde_json::from_str::<Value>(&first.text) {
                    return Ok(parsed_json);
                } else {
                    return Ok(Value::String(first.text));
                }
            } else {
                return Ok(Value::Null);
            }
        }

        // Return raw result if not conforming to CallToolResult wrapper
        Ok(result_val)
    }

    /// Discover remote MCP tools and synthesize them as native [`Reagent`] instances.
    ///
    /// The resulting `Arc<dyn Reagent>` items can be provided directly to Rubedo `Artificer` instances.
    pub async fn discover_reagents(self: &Arc<Self>) -> Result<Vec<Arc<dyn Reagent>>, McpError> {
        let tools = self.list_tools().await?;
        let mut reagents: Vec<Arc<dyn Reagent>> = Vec::with_capacity(tools.len());

        for tool in tools {
            // Leak strings to produce &'static str required by the Reagent trait
            let static_name: &'static str = Box::leak(tool.name.into_boxed_str());
            let static_desc: &'static str =
                Box::leak(tool.description.unwrap_or_default().into_boxed_str());

            let dynamic_reagent = DynamicMcpReagent {
                client: Arc::clone(self),
                name: static_name,
                description: static_desc,
                input_schema: tool.input_schema,
                output_schema: serde_json::json!({
                    "type": "object",
                    "description": "Dynamic output payload from remote MCP tool"
                }),
            };

            reagents.push(Arc::new(dynamic_reagent));
        }

        Ok(reagents)
    }

    /// Gracefully shutdown the client, terminating the receiver task and closing the transport.
    pub async fn close(&self) -> Result<(), McpError> {
        self._receiver_task.abort();
        self.transport.close().await
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self._receiver_task.abort();
    }
}


/// Dynamic Reagent wrapping an external MCP tool invocation over JSON-RPC.
pub struct DynamicMcpReagent {
    client: Arc<McpClient>,
    name: &'static str,
    description: &'static str,
    input_schema: Value,
    output_schema: Value,
}

#[async_trait]
impl Reagent for DynamicMcpReagent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    fn output_schema(&self) -> Value {
        self.output_schema.clone()
    }

    async fn transmute(&self, input: Value) -> Result<Value, ReagentError> {
        self.client
            .call_tool(self.name, input)
            .await
            .map_err(Into::into)
    }
}
