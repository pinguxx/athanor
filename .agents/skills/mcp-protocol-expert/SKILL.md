---
name: mcp-protocol-expert
description: >-
  Authoritative expert on the Model Context Protocol (MCP 2024-11-05+) and JSON-RPC 2.0.
  Use when designing, auditing, or testing rubedo-mcp, tool discovery (tools/list),
  tool execution (tools/call), or stdio/SSE transport channels.
---

# MCP Protocol Expert (Model Context Protocol Guide)

A specialized skill for building, verifying, and testing bidirectional Model Context Protocol (MCP) integrations within `rubedo-mcp`. Ensures seamless interoperability with Claude Desktop, Cursor, and remote MCP tool ecosystems.

---

## 🎯 Bidirectional MCP Architecture

```text
External Client (e.g., Claude Desktop, Cursor)
        │
        ├── [JSON-RPC 2.0 via Stdio or SSE]
        ▼
   McpServer (rubedo-mcp)
        │
        ├── Discovers & registers native #[reagent] tools
        ├── Exposes tool schemas via `tools/list`
        └── Dispatches execution via `tools/call` -> `Reagent::transmute()`
-------------------------------------------------------------------------
   McpClient (rubedo-mcp)
        │
        ├── Connects to external MCP servers (stdio or SSE)
        ├── Fetches remote schemas via `tools/list`
        └── Wraps remote capabilities as native `Arc<dyn Reagent>` for Artificers
```

---

## 📜 Core JSON-RPC 2.0 Message Types

### 1. Handshake (`initialize`)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": { "tools": {} },
    "clientInfo": { "name": "rubedo-mcp-client", "version": "0.1.0" }
  }
}
```

### 2. Discover Tools (`tools/list`)
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "get_stock_price",
        "description": "Fetches latest quote",
        "inputSchema": {
          "type": "object",
          "properties": {
            "symbol": { "type": "string" }
          },
          "required": ["symbol"]
        }
      }
    ]
  }
}
```

### 3. Invoke Tool (`tools/call`)
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_stock_price",
    "arguments": { "symbol": "AAPL" }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"symbol\":\"AAPL\",\"price\":150.25}"
      }
    ]
  }
}
```

---

## 📋 Technical References
- [MCP Spec Cheatsheet](./references/mcp_spec_cheatsheet.md) — Complete protocol specification and error codes.
- [Client Interoperability Checklist](./references/interop_checklist.md) — Compatibility checklist for Claude Desktop & Cursor.
