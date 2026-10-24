# 🔬 50-Point Rubedo Adversarial Review Checklist

This checklist provides deep-inspection questions across five mission-critical engineering domains for the **Rubedo** agentic framework.

---

## 1. Rust Primitives & Macro Synthesizers (10 Points)

1. **Zero Lifetime Annotations**: Do public macros (`#[reagent]`, `opus!`, `artificer!`) mandate `'static + Send + Sync` using owned value transfers and `Arc` envelopes, eliminating all lifetime annotations for autonomous agent authoring?
2. **Panic Freedom**: Does the code contain any unvalidated `.unwrap()`, `.expect()`, `panic!()`, or unreachable branches in non-test crate paths?
3. **Integer & Float Arithmetic**: Can token counters, budget meters, or step counters overflow or divide by zero? (Use `saturating_add` or checked arithmetic).
4. **Lock Scope & Concurrency**: Are `std::sync::Mutex` or `tokio::sync::RwLock` guards held across `.await` points or over lengthy I/O operations?
5. **Async Runtime Blocking**: Are CPU-intensive operations (WASM compilation, embedding generation, text splitting) spawned on `tokio::task::spawn_blocking` rather than blocking worker threads?
6. **Error Propagation**: Are all errors classified using the `ReagentError` hierarchy (`InvalidInput`, `Recoverable` with guidance, `Fatal`) or derived from `thiserror::Error`?
7. **Resource Cleanup**: Are SQLite connection handles, file descriptors, and WASM memory buffers guaranteed to release on early-return or error exit?
8. **Clone & Allocation Overhead**: Are heavy AST nodes, JSON payloads, or vector buffers cloned unnecessarily in hot loops?
9. **AST Parsing Edge Cases**: Does `syn` parsing in `rubedo-macros` gracefully handle empty parameter lists, doc comments, optional types, and edge-case identifiers?
10. **Trait Invariants**: Do custom implementations of `Reagent` and `Vessel` uphold `'static + Send + Sync` contracts under all generics?

---

## 2. Durable Workflows & SQLite Event Sourcing (10 Points)

11. **Atomic Checkpointing**: Is every Vessel execution transition (pre-execution snapshot, post-execution result) wrapped in an atomic SQLite transaction in `rubedo-crucible`?
12. **Resume Token Entropy**: Are resume tokens minted using cryptographically secure UUIDv4 strings rather than sequential or guessable identifiers?
13. **Single-Use Token Invariant**: Is a resume token atomically consumed from `suspended_tokens` upon invocation of `OpusEngine::resume()`, preventing double-resumption replay attacks?
14. **State Deserialization Safety**: Is workflow state JSON deserialized safely with strict schema validation, rejecting malformed historical snapshots?
15. **DAG Cycle Prevention**: Does `OpusBuilder` validate that workflow topologies are strictly directed and acyclic before allowing execution?
16. **Crash-Recovery Consistency**: If the host process is terminated abruptly during step execution, does `CrucibleEngine` leave the database in a cleanly recoverable state?
17. **Migration Versioning**: Are SQLite database schema migrations embedded using `sqlx::migrate!()` with explicit backward compatibility?
18. **Time-Travel Safety**: Can past Vessel events be replayed without side-effect mutations affecting unrelated active runs?
19. **Index Coverage**: Are query filters (`run_id`, `vessel_id`, `token`) backed by SQLite indexes to prevent table scans under heavy event volume?
20. **Payload Size Clamping**: Are step payload sizes bounded to prevent SQLite database bloat or memory exhaustion during event loading?

---

## 3. Sandboxed WASM Isolation & Host-Guest Memory (10 Points)

21. **Fuel Metering Enforcement**: Does `AlembicSandbox` mandate instruction fuel consumption on all executions, aborting infinite loops within $<15$ms?
22. **Linear Memory Ceilings**: Is the guest WASM memory strictly bounded (e.g. max 64MB) to prevent guest modules from consuming all host RAM?
23. **Zero-Access WASI**: Are filesystem, network sockets, and environment variables excluded by default from the WASI context?
24. **Guest Buffer Bounds Checking**: Does the host memory exchange reader strictly validate guest pointer and length offsets before reading linear memory?
25. **Host Panic Immunity**: If a guest WASM module traps (`unreachable`, stack overflow, memory out-of-bounds), does the host gracefully return `AlembicError::Trap` without crashing?
26. **Deterministic Aborts**: Are fuel exhaustion errors explicitly categorized as recoverable or fatal based on caller configuration?
27. **Module Compilation Caching**: Are frequently invoked WASM modules compiled once and shared across stores rather than recompiled per invocation?
28. **Thread Safety of Sandbox**: Is `AlembicSandbox` safe to share across threads (`Send + Sync + 'static`)?
29. **Reagent Trait Parity**: Does `WasmReagent` implement the full `rubedo_core::reagent::Reagent` contract faithfully?
30. **No Unaudited Unsafe**: Is all memory pointer manipulation inside `rubedo-alembic` thoroughly audited with explicit `// SAFETY` justification comments?

---

## 4. MCP Gateway & Interoperability Boundaries (10 Points)

31. **JSON-RPC 2.0 Compliance**: Does `rubedo-mcp` strictly follow the JSON-RPC 2.0 specification for requests, responses, notifications, and error codes?
32. **Tool Schema Parity**: Does `McpServer` expose the exact JSON Schema generated by `schemars` from internal `Reagent` instances?
33. **Remote Tool Sanitization**: Does `McpClient` validate remote MCP tool definitions before wrapping them as native `dyn Reagent` instances?
34. **Channel Transport Isolation**: Does stdio transport handle newline framing, buffer flushes, and EOF cleanly without hanging the process?
35. **SSE Transport Disconnects**: Does Server-Sent Events transport detect client disconnects and cleanly reclaim resources?
36. **Input Type Coercion**: Are input parameters validated against the declared JSON Schema before invoking `reagent.transmute()`?
37. **Error Masking**: Do internal server errors avoid leaking host file system paths or database credentials to remote MCP clients?
38. **Deadlock-Free Duplex**: Are duplex communications between McpClient and McpServer free of lock-step buffering deadlocks?
39. **Timeouts on Remote Calls**: Are external MCP tool invocations protected by configurable execution timeouts?
40. **Concurrent Invocations**: Can multiple MCP clients invoke distinct reagents concurrently without cross-contamination?

---

## 5. Atelier Visualizer, RAG & Testing Rigor (10 Points)

41. **Single-Binary Asset Embedding**: Are frontend UI assets bundled cleanly via `rust-embed` with correct MIME types and fallback routing?
42. **SSE Broadcast Channel Backpressure**: Does the Tokio broadcast channel in `rubedo-atelier` handle lagging subscribers gracefully without crashing the server?
43. **In-Process RAG Footprint**: Does `rubedo-rag` maintain an idle memory footprint $<40$MB using `fastembed-rs` and embedded `lancedb`?
44. **Semantic Chunking Consistency**: Do text splitters maintain chunk overlap and avoid cutting markdown code blocks or table headers midway?
45. **Failure Path Coverage**: Do automated tests assert error responses (e.g. `InvalidInput`, `Recoverable`, `FuelExhausted`), not just happy paths?
46. **Regression Guard**: Is every bug fix accompanied by a regression test replicating the exact reported failure?
47. **Lint & Formatting Zero-Tolerance**: Does the PR pass `cargo clippy --workspace --all-targets -- -D warnings` with zero warnings?
48. **Clean Documentation**: Does `cargo doc --workspace --no-deps` compile without broken links or missing docs?
49. **Coverage Threshold**: Does `cargo llvm-cov --workspace` verify $\ge 80\%$ line coverage across all non-test code?
50. **No Prohibited Panics**: Does the workspace strictly forbid unhandled panics and adhere to the Alchemical Lexicon?
