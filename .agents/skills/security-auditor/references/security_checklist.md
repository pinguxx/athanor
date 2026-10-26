# Athanor Security Audit Checklist & Verification Guide

This checklist provides a systematic methodology for verifying the security posture of the **Athanor** agentic runtime across its Rust crates and embedded visualizer.

---

## 1. WASM Sandboxing & In-Process Isolation (`athanor-alembic`)

- [ ] **Instruction Fuel Metering**:
  - `wasm_config.consume_fuel(true)` is enabled when constructing the Wasmtime engine.
  - Every execution sets `store.set_fuel(limit)` prior to calling guest functions.
  - Infinite loops or high-iteration code trigger `Trap::OutOfFuel` and map to `AlembicError::FuelExhausted` within $<15$ms.
- [ ] **Memory Envelopes**:
  - Linear memory max allocation is capped (default: 64MB).
  - Guest pointers and buffer lengths are checked before host reads/writes.
  - Out-of-bounds pointer reads from guest memory never trigger host panics or segfaults.
- [ ] **Capability Stripping (Zero-Access WASI)**:
  - Untrusted WASM modules have no access to host filesystem, network sockets, or environment variables.
  - Standard input/output are isolated to memory buffers.

---

## 2. MCP Gateway & JSON-RPC Protocols (`athanor-mcp`)

- [ ] **Protocol Boundary & Serialization**:
  - Complies strictly with JSON-RPC 2.0 schema format.
  - Input JSON payloads are validated against the declared JSON Schema before invocation.
  - Error messages returned to clients do not reveal internal file paths or system credentials.
- [ ] **Transport Layer Security**:
  - Stdio transport handles malformed JSON without hanging the reader thread.
  - Server-Sent Events (SSE) connections authenticate or restrict origins to local development clients.
- [ ] **Dynamic Tool Registration**:
  - Remote tools discovered via `tools/list` are sanitized before mapping to `dyn Reagent`.
  - Tool names cannot overwrite internal reserved framework commands.

---

## 3. Workflow State Durability & Tokens (`athanor-crucible` & `athanor-opus`)

- [ ] **Resume Token Generation & Security**:
  - Resume tokens are minted using `uuid::Uuid::new_v4()` (128-bit cryptographic entropy).
  - Resume tokens cannot be guessed, enumerated, or forged.
- [ ] **Single-Use Token Enforcement**:
  - Resumption consumes the token from SQLite in an atomic transaction (`DELETE FROM suspended_tokens WHERE token = ?`).
  - Attempting to resume with an already-consumed token returns an explicit error (`CrucibleError::TokenNotFound`).
- [ ] **SQL Injection Defense**:
  - All queries in `CrucibleEngine` use `sqlx` parameterized queries (`?`).
  - Zero dynamic SQL concatenation for tables, columns, or data.

---

## 4. Rust Invariants & Code Hygiene (`athanor-core`)

- [ ] **Panic-Free Production Paths**:
  - Zero `.unwrap()` or `.expect()` calls in non-test crate sources (`crates/*/src/`).
  - Array and slice indexing is checked or guarded against out-of-bounds panics.
- [ ] **Safe Rust Enforcement**:
  - Zero unannotated `unsafe` blocks. Any `unsafe` block must include a descriptive `// SAFETY:` comment.
- [ ] **Thread Safety & Send/Sync**:
  - All types passing through `Opus`, `Reagent`, and `Artificer` satisfy `'static + Send + Sync`.
  - No lock guards (`MutexGuard`, `RwLockReadGuard`) held across `.await` points.
- [ ] **Error Model**:
  - Errors derive `thiserror::Error` and categorize failures into `InvalidInput`, `Recoverable` with guidance, and `Fatal`.

---

## 5. Embedded Atelier Dev Server (`athanor-atelier`)

- [ ] **Network Binding**:
  - Server binds to `127.0.0.1` by default, preventing unintended exposure on local networks.
- [ ] **HTTP Security Headers**:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Content-Security-Policy: default-src 'self';`
- [ ] **Input & Payload Limits**:
  - Endpoints (`POST /api/opus/:run_id/resume`) limit incoming payload size to prevent memory exhaustion.
