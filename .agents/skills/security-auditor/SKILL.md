---
name: security-auditor
description: >-
  Audits the security posture, memory boundaries, and execution containment of Athanor.
  Use this skill whenever asked to review security, conduct a vulnerability assessment,
  audit WASM sandboxing, inspect MCP gateway boundaries, check SQLite state durability,
  or evaluate panic prevention (unwrap/expect) and OWASP compliance.
---

# Security Auditor (Athanor Agent Security)

A specialized offensive-and-defensive application security auditing skill designed to discover, demonstrate, and remediate vulnerabilities across the **Athanor** agentic runtime: WASM sandbox isolation, MCP JSON-RPC protocols, SQLite event store persistence, safe Rust invariants, and the embedded Atelier dev server.

---

## 🎯 Security Auditor Philosophy & Approach

1. **Zero Trust for Agent Inputs & Dynamic Code**: Assume LLM completions are non-deterministic, third-party Reagents are potentially malicious, and remote MCP servers can craft malformed JSON-RPC payloads. Validate, constrain, and sandbox at every layer.
2. **Deterministic Containment**: Untrusted or model-generated tools must never have raw access to host memory, filesystem, or network. Every WASM execution must be constrained by strict memory envelopes and instruction fuel limits.
3. **Exploitability & Concrete PoCs**: Accompany security findings with concrete proof-of-concept payloads explaining *how* an adversary or rogue model output could bypass boundaries or crash the host runtime.
4. **Actionable Remediation**: Provide production-ready, hardened code diffs with defense-in-depth mitigations (e.g., parameterization, fuel accounting, type constraints, token rotation).

---

## 🛡️ Core Vulnerability Vectors & Checklists

### 1. In-Process WASM Isolation (`athanor-alembic`)
- **Fuel Accounting**:
  - Verify `wasm_config.consume_fuel(true)` is unconditionally set.
  - Ensure every execution sets `store.set_fuel(limit)` prior to module invocation.
  - Verify that infinite loops or CPU exhaustion attacks abort deterministically within $<15$ms without crashing the host.
- **Linear Memory Envelopes**:
  - Ensure guest linear memory allocation is capped (default: 64MB).
  - Verify host linear memory access validates pointers and buffer lengths before copying data to avoid host memory corruption.
- **Capability Isolation**:
  - Ensure WASI filesystem access, network sockets, and process environment variables are excluded from untrusted guest modules.

### 2. Bidirectional MCP Gateway (`athanor-mcp`)
- **JSON-RPC 2.0 Request Validation**:
  - Strict parsing and validation of incoming JSON-RPC payloads.
  - Ensure remote MCP tool schemas are validated before being registered as native dynamic Reagents.
- **Command & Path Injection**:
  - Ensure tool names and parameters are never interpolated into shell commands or unsanitized filesystem paths.
- **Resource Exhaustion on Duplex Channels**:
  - Enforce timeouts and payload size limits on incoming/outgoing stdio and SSE messages.

### 3. Workflow State Durability & Token Security (`athanor-crucible` & `athanor-opus`)
- **Resume Token Forgery**:
  - Resume tokens must be cryptographically secure UUIDv4 strings.
  - Verify tokens cannot be guessed or forged by external actors.
- **Atomic Single-Use Token Invariant**:
  - Tokens in `suspended_tokens` must be consumed atomically in a SQLite transaction.
  - Verify that a token cannot be replayed or used to resume execution multiple times.
- **SQL Injection Defense**:
  - All SQLite database queries must use `sqlx` parameterized prepared statements (`?` / `$1`).
  - Zero dynamic SQL string concatenation for table names, column names, or values.

### 4. Rust Invariants & Panic Prevention (`athanor-core`)
- **Zero Panics**:
  - Strictly forbid `.unwrap()` and `.expect()` calls in non-test crate sources.
  - Ensure all external and JSON deserialization failures map gracefully to `ReagentError` or `AthanorError`.
- **100% Safe Rust**:
  - Maintain zero unannotated `unsafe` blocks. Any unavoidable `unsafe` must include a detailed `// SAFETY:` rationale.
- **Memory & Concurrency Bounds**:
  - Ensure types passing through `Opus`, `Reagent`, and `Artificer` satisfy `'static + Send + Sync`.

### 5. Embedded Atelier Dev Server (`athanor-atelier`)
- **HTTP Security Headers**:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Content-Security-Policy: default-src 'self'; ...`
- **CORS & Network Binding**:
  - Atelier dev server must bind to `127.0.0.1` by default, never listening on `0.0.0.0` without explicit user flag.
  - Resume endpoint (`POST /api/opus/:run_id/resume`) must validate payload schema and token.

---

## 🛠️ Security Audit Automation

Execute the automated security and invariant audit script:
```powershell
powershell -ExecutionPolicy Bypass -File .agents/skills/security-auditor/scripts/audit_security.ps1
```

For exhaustive audit checks and risk classifications, consult:
- [Athanor Security Checklist](./references/security_checklist.md)
- [Athanor Vulnerability Catalog](./references/vulnerability_catalog.md)
