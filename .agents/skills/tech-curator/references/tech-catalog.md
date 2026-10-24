# Alchemical Systems Engineering Catalog

A curated reference catalog of foundational systems engineering themes, architectural tradeoffs, and performance analyses guiding the **Rubedo** framework.

---

## 1. Embedded SQLite vs Distributed Microservices
- **Core Thesis**: Running SQLite in write-ahead log (WAL) mode embedded directly in a compiled Rust binary easily processes 80,000+ reads/sec with sub-millisecond $p99$ latencies on modern NVMe drives.
- **Complexity Penalty**: Replacing embedded SQLite with a Kubernetes cluster running Postgres, Redis, and a Kafka message bus increases operational attack surface by $10\times$ and latency by $5\times$ to $20\times$ due to network serialization and TCP handshakes.
- **Rubedo Application**: `rubedo-crucible` uses embedded SQLite via `sqlx` to provide event-sourced durability, pause/resume checkpointing, and time-travel replay without external infrastructure.

---

## 2. In-Process WASM Isolation vs Heavyweight Docker Containers
- **Core Thesis**: Spin-up latency for a Linux container is 200–800ms with a baseline 50MB+ memory overhead per container. In contrast, an in-process Wasmtime WebAssembly instance instantiates in $<1$ms with a sub-megabyte memory footprint.
- **Instruction Fuel Tracking**: Wasmtime enables instruction-level fuel metering, deterministically terminating infinite loops and CPU-hog attacks in $<15$ms without OS thread killing.
- **Rubedo Application**: `rubedo-alembic` isolates untrusted and model-generated Reagents in WebAssembly envelopes with 64MB memory caps and 100M instruction fuel limits.

---

## 3. Zero Lifetime Annotations & Autonomous Code Generation
- **Core Thesis**: Rust's borrow checker guarantees memory safety without garbage collection, but complex lifetime parameters (`'a`, `'b: 'a`) are notorious for causing compiler errors when code is synthesized dynamically by LLMs.
- **Value Semantics & Arc Envelopes**: Designing macro synthesizers around owned value transfers (`serde_json::Value`) and thread-safe envelopes (`Arc<T>`, `'static + Send + Sync`) completely eliminates lifetime annotations from consumer code while preserving 100% memory safety.
- **Rubedo Application**: `rubedo-macros` (`#[reagent]`) synthesizes JSON Schema and input structures without requiring a single lifetime parameter.

---

## 4. In-Process Vector Embeddings (<40MB) vs Cloud Vector Databases
- **Core Thesis**: Small, specialized embedding models (e.g. `BGESmallENV15`) running via ONNX Runtime in-process achieve $>0.70$ cosine similarity retrieval accuracy with sub-20ms latency and zero cloud roundtrips.
- **Zero Infrastructure Footprint**: Combining in-process FastEmbed with local embedded LanceDB achieves local semantic retrieval within a resident memory budget $<40$MB, eliminating Pinecone, Qdrant, or Weaviate network dependencies for edge and local agent runtimes.
- **Rubedo Application**: `rubedo-rag` exposes `SpagyricIndex` directly within the compiled agent binary.

---

## 5. Bidirectional Model Context Protocol (MCP) as Universal Interop
- **Core Thesis**: Custom plugin APIs fragment developer ecosystems. Standardizing on JSON-RPC 2.0 over stdio and SSE allows any agent or tool to act simultaneously as an MCP client and an MCP server.
- **Rubedo Application**: `rubedo-mcp` exposes native Reagents to Claude Desktop, Cursor, and external tools while consuming remote MCP endpoints as first-class `dyn Reagent` instances.
