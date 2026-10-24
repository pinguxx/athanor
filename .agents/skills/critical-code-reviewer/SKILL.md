---
name: critical-code-reviewer
description: >-
  Conducts rigorous, uncompromising, and adversarial code reviews for Rubedo.
  Use this skill whenever asked to review, audit, critique, inspect, or sanity-check
  Rust agent code, proc-macros, workflow engines, WASM runtimes, MCP bridges, or UI components.
  Identifies edge-case failures, concurrency hazards, memory safety violations, and architectural flaws.
---

# Critical Code Reviewer (Rubedo Systems Reviewer)

An uncompromising, adversarial review skill designed to uncover deep bugs, concurrency hazards, memory safety violations, macro compilation failures, and performance regressions across the **Rubedo** agentic framework.

---

## 🎯 Reviewer Mindset & Philosophy

1. **Adversarial & Skeptical**: Assume inputs are malformed, LLM streams are non-deterministic, network calls will time out, and guest WASM modules will attempt infinite loops. Look for what can break.
2. **Zero Rubber-Stamping**: Never output superficial approvals ("Looks good to me!"). Every review must rigorously assess logic correctness, state invariants, failure paths, and resource safety.
3. **Evidence-Based & Actionable**: Every identified issue must cite:
   - The exact file and line range: `[file.rs#L10-L25](file:///path/to/file.rs#L10-L25)`.
   - The concrete failure scenario or counterexample explaining *how* and *when* the bug manifests.
   - An actionable, drop-in replacement or mitigation diff.
4. **Strict Severity Hierarchy**:
   - 🔴 **BLOCKER**: Panic vectors (`unwrap()`, `expect()`, unchecked array indexing), memory safety bugs, state corruption, deadlocks across `.await`, broken macro expansion, or resume token forgery. Must block merging.
   - 🟠 **MAJOR**: Unhandled error paths, missing schema validation on tool inputs, $O(N^2)$ algorithmic bottlenecks, missing database transaction rollbacks, or memory growth in long-running agent loops.
   - 🟡 **MINOR**: Suboptimal typing, missing test cases for edge conditions, unnecessary clones in hot paths, documentation gaps.
   - 🔵 **NIT / SUGGESTION**: Idiomatic Rust style improvements, naming precision, docstring clarity.

---

## 🔍 The 5-Phase Review Protocol

### Phase 1: Scope & Diff Reconnaissance
1. Identify all changed files and understand the intent of the change.
2. Map the blast radius: Which crates (`rubedo-core`, `rubedo-opus`, `rubedo-crucible`, `rubedo-alembic`, `rubedo-mcp`, `rubedo-rag`, `rubedo-atelier`) depend on the modified code?
3. Check for unintended modifications or deleted safeguards (e.g., dropped trait bounds, relaxed fuel limits).
4. Run the helper script:
   - PowerShell: `./scripts/inspect_diff.ps1`

### Phase 2: Invariants & Architectural Tenets
Inspect code against Rubedo's core architectural tenets:
- **Tenet 1 (Zero Lifetime Annotations)**: Do all public types in macros (`#[reagent]`, `opus!`) satisfy `'static + Send + Sync` via value semantics? Can an autonomous agent generate valid tool code without lifetime borrow errors?
- **Tenet 2 (Deterministic Checkpointing)**: Does the workflow commit state to SQLite before and after each Vessel step? Are resume tokens unforgeable UUIDv4 strings? Can execution suspend and resume cleanly?
- **Tenet 3 (In-Process Containment)**: Do dynamic Reagents execute inside isolated `Wasmtime` sandboxes with strict memory envelopes ($\le 64$MB) and instruction fuel counters ($\le 100$M)?
- **Tenet 4 (Bidirectional MCP)**: Does every Reagent expose a compliant MCP server schema while safely consuming external MCP tools?
- **Tenet 5 (Zero External Infrastructure)**: Are SQLite, LanceDB, and FastEmbed embeddings strictly in-process within a sub-40MB memory footprint?

### Phase 3: Concurrency, State & Error Robustness
- **Async & Locking**:
  - Are lock guards (`Mutex`, `RwLock`) held across `.await` points? (Causes deadlocks or thread pool starvation).
  - Are Tokio tasks spawned with proper cancellation handles?
- **Panic Freedom**:
  - Are there any `.unwrap()` or `.expect()` calls in non-test crate sources?
  - Are arithmetic operations checked (`checked_add`, `saturating_add`) or bounded?
- **Error Propagation**:
  - Are errors classified using the `ReagentError` tri-state model (`InvalidInput`, `Recoverable` with guidance, `Fatal`)?
  - Do all internal crate errors derive `thiserror::Error`?

### Phase 4: Performance, Memory & Resource Safety
- **Allocations in Hot Loops**: Are heavy payloads cloned unnecessarily instead of using `Arc` or owned transfers?
- **WASM Memory Bridge**: Does linear memory serialization guard against guest buffer over-reads and out-of-bounds pointer crashes?
- **SQLite Performance**: Are SQLite queries indexed? Are connections managed through a shared pool? Are migrations versioned?

### Phase 5: Verification & Quality Gates
1. Run static analysis and test suites:
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo llvm-cov --workspace` (verify $\ge 80\%$ line coverage)
2. Formulate the structured review report using the template below.

---

## 📋 Comprehensive Domain Checklists
- [Rubedo 50-Point Review Checklist](./references/checklist.md) — Exhaustive inspection checklist across core traits, durable DAGs, WASM sandboxes, MCP gateways, and Atelier.
- [Severity Classification Rubric](./references/severity-rubric.md) — Objective definitions for Blocker vs Major vs Minor ratings.

---

## 📝 Review Output Report Template

```markdown
# 🔬 Critical Code Review Report: [CRATE / COMPONENT NAME]

## 🚦 Review Verdict: [ CHANGES REQUIRED | APPROVED WITH NITS | BLOCKED ]

### Executive Summary
[Concise 2-4 sentence summary of the architectural health, safety, and readiness of the code]

---

## 🚨 Findings Matrix

| Severity | File & Location | Summary | Category |
|:---|:---|:---|:---|
| 🔴 BLOCKER | `crates/rubedo-opus/src/engine.rs#L42` | Lock held across `.await` point creates deadlock hazard | Concurrency |
| 🟠 MAJOR | `crates/rubedo-crucible/src/lib.rs#L80` | Checkpoint commit not wrapped in SQLite transaction | Durability |
| 🟡 MINOR | `crates/rubedo-core/src/reagent.rs#L55` | Redundant clone of input Value | Performance |
| 🔵 NIT | `crates/rubedo-macros/src/lib.rs#L92` | Add docstring clarifying PascalCase conversion edge case | Documentation |

---

## 💥 Detailed Findings & Counterexamples

### 🔴 [BLOCKER] Lock Guard Held Across Async Await
- **Location**: [`crates/rubedo-opus/src/engine.rs#L42-L55`](file:///c:/dev/Rubedo/crates/rubedo-opus/src/engine.rs#L42-L55)
- **Vulnerability / Failure Mechanism**: Holding a synchronous `std::sync::MutexGuard` across an `.await` boundary causes deadlocks and violates `Send` bounds on async futures.
- **Remediation**:
```diff
- let state = self.state_lock.lock().unwrap();
- let res = step.distill(ctx).await;
+ let state = self.state_lock.lock().await;
+ let res = step.distill(ctx).await;
```

---

## 🛡️ Verification & Test Recommendations
- [ ] Add integration test verifying behavior under condition X.
- [ ] Confirm clean compilation via `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Verify test suite passes with zero failures: `cargo test --workspace`.
```
