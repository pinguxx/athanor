# Phase 4: Bidirectional Model Context Protocol (MCP) Gateway

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-mcp`
- **Primary Goal**: Provide a bidirectional MCP bridge compliant with the Model Context Protocol specification. Rubedo can simultaneously expose its native Reagents and Artificers as an MCP server to external consumers (like Claude Desktop or Cursor), and consume external MCP servers as native `Arc<dyn Reagent>` tools.

---

## 2. Detailed Task Breakdown

### Task 4.1: Scaffold `rubedo-mcp` & JSON-RPC Protocol Types
- Create `crates/rubedo-mcp/Cargo.toml` with dependencies:
  - `rubedo-core`, `serde`, `serde_json`, `tokio`, `futures`, `async-trait`, `thiserror`, `tracing`
- In `crates/rubedo-mcp/src/protocol.rs`:
  - Define JSON-RPC 2.0 request/response structures:
    - `JsonRpcRequest { jsonrpc: "2.0", id, method, params }`
    - `JsonRpcResponse { jsonrpc: "2.0", id, result, error }`
  - Define MCP schema messages:
    - `InitializeRequest`, `InitializeResult`
    - `ListToolsRequest`, `ListToolsResult`, `ToolDefinition`
    - `CallToolRequest`, `CallToolResult`, `Content`

### Task 4.2: Transport Layer Abstractions
- In `crates/rubedo-mcp/src/transport/mod.rs`:
  - Trait `Transport: Send + Sync`:
    - `async fn send(&self, msg: String) -> Result<(), TransportError>;`
    - `async fn receive(&self) -> Result<Option<String>, TransportError>;`
  - Implement `StdioTransport` (stdin/stdout pipe).
  - Implement `ChannelTransport` (in-memory `tokio::sync::mpsc` duplex for zero-overhead testing).

### Task 4.3: Implement `McpServer`
- In `crates/rubedo-mcp/src/server.rs`:
  - Maintain a registry of `Arc<dyn Reagent>`.
  - Handle `initialize`: returns server capabilities and protocol version.
  - Handle `tools/list`: enumerates registered reagents, mapping their `input_schema()` to JSON-RPC tools list.
  - Handle `tools/call`: extracts tool name and parameters, invokes `reagent.transmute(params)`, and returns the JSON result.

### Task 4.4: Implement `McpClient`
- In `crates/rubedo-mcp/src/client.rs`:
  - Connect to an MCP transport.
  - Handshake via `initialize`.
  - Fetch tools via `tools/list`.
  - Dynamically synthesize `DynamicMcpReagent` instances implementing `rubedo_core::reagent::Reagent` for each discovered tool.
  - Calling `transmute(input)` on the `DynamicMcpReagent` forwards a `tools/call` JSON-RPC message over the transport and returns the parsed result.

---

## 3. Verification & Acceptance Criteria

### Integration Test
Create `crates/rubedo-mcp/tests/bidirectional_mcp_test.rs`:
1. Define a native tool:
   - Tool `CalculatorReagent`: adds two numbers `{"a": i64, "b": i64}`.
2. Spin up `McpServer` with `CalculatorReagent`.
3. Create an in-memory duplex channel and connect `McpClient`.
4. Perform discovery:
   - Call `client.list_tools().await`.
   - Assert discovered list contains `"calculator"` with the correct schema.
5. Perform execution:
   - Execute the tool via client: `client.call_tool("calculator", json!({"a": 15, "b": 27})).await`.
   - Assert result is `json!({"sum": 42})`.

### Execution Command
```bash
cargo test -p rubedo-mcp
```
