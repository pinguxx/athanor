# athanor-mcp

Bidirectional Model Context Protocol (MCP 2024-11-05+) bridge for the Athanor agentic framework.

## Features
- **Bidirectional Architecture**: Operate simultaneously as an MCP Server exposing native Reagents and an MCP Client consuming external tools.
- **Dynamic Reagent Synthesis**: External MCP tools discovered via `tools/list` are transparently wrapped into native Athanor `Reagent` instances.
- **Transports**: Stdio transport for external CLI/desktop AI agents (Claude Desktop, Cursor) and in-memory duplex channel transport for embedded unit testing.
