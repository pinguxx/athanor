# MCP Specification Cheat Sheet & Error Codes

## Standard JSON-RPC 2.0 Error Codes

| Code | Message | Meaning in MCP Context |
|:---|:---|:---|
| `-32700` | Parse error | Invalid JSON was received by the server. |
| `-32600` | Invalid Request | The JSON sent is not a valid Request object. |
| `-32601` | Method not found | The method does not exist (e.g. unknown tool or method). |
| `-32602` | Invalid params | Tool arguments failed JSON Schema validation. |
| `-32603` | Internal error | Internal server or execution error. |

---

## Transport Guidelines

1. **Stdio Transport**:
   - Each JSON-RPC message is sent as a single line terminated by a newline (`\n`).
   - Standard output (`stdout`) MUST only contain valid JSON-RPC lines. Any debug logging must be directed to `stderr`.
2. **SSE (Server-Sent Events) Transport**:
   - Client sends HTTP GET to establish SSE stream.
   - Client sends HTTP POST requests with JSON-RPC bodies to the designated endpoint.
   - Server streams responses over the active SSE connection with event type `message`.
