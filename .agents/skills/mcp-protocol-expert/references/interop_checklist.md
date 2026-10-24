# Claude Desktop & Cursor MCP Interoperability Checklist

To ensure `rubedo-mcp` connects seamlessly to desktop AI clients:

- [ ] **No stdout Pollution**: Ensure tracing or print statements in `rubedo-mcp` write to `stderr`, keeping `stdout` reserved exclusively for newline-delimited JSON-RPC messages.
- [ ] **Schema Compatibility**: Ensure `inputSchema` matches JSON Schema draft-07 standard outputted by `schemars`.
- [ ] **Content Format**: Tool return values in `tools/call` must be wrapped inside `content: [{ "type": "text", "text": "..." }]`.
- [ ] **Process Lifecycle**: McpServer must cleanly exit when standard input (`stdin`) reaches EOF.
