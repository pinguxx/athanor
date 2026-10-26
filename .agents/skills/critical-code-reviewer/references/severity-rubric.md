# ⚖️ Severity Classification Rubric for Athanor

Use this standardized rubric to ensure consistent, objective classification of code review findings across the **Athanor** agentic framework.

---

## 🔴 BLOCKER (P0 / Critical)
**Criteria**: Code that compromises security, causes state corruption, crashes the agent runtime, or violates core architectural tenets. **Merge is blocked until resolved.**

### Concrete Examples:
- Lifetime annotations in public macros (`#[reagent]`, `opus!`), violating Tenet 1 (Zero Lifetime Annotations).
- Panic on untrusted input in Rust (`.unwrap()`, `.expect()`, out-of-bounds slice indexing) in non-test crates.
- Memory leak or unmetered execution in WASM sandbox (missing fuel checks in `AlembicSandbox`).
- Resume token forgery or non-atomic token consumption in `CrucibleEngine`.
- Lock guard (`Mutex`, `RwLock`) held across `.await` points, causing thread pool deadlocks.
- Plaintext credential, secret, or private key hardcoded into source or committed to Git.
- Failing workspace tests or compiler/clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`.

---

## 🟠 MAJOR (P1 / High)
**Criteria**: Flaws in architecture, performance, error handling, or state synchronization that impair agent reliability, scalability, or developer experience. **Resolution strongly required prior to release.**

### Concrete Examples:
- Unindexed SQLite queries or $O(N^2)$ in-memory loops over potentially unbounded event logs.
- Missing schema validation on tool inputs before calling `reagent.transmute()`.
- Unhandled error states or silent error swallowing (`let _ = ...` without logging or telemetry event).
- Missing SQLite transaction boundaries across multi-table mutating operations in `CrucibleEngine`.
- Memory growth in long-running agent loops due to unbounded `AthanorContext` message accumulation without trimming.
- External infrastructure dependency added (e.g., PostgreSQL, Redis), violating Tenet 5.

---

## 🟡 MINOR (P2 / Medium)
**Criteria**: Code smells, non-critical edge cases, type safety degradation, or missing test coverage that reduces maintainability and robustness.

### Concrete Examples:
- Missing unit tests for edge-case boundaries (e.g., zero-parameter reagents, empty query strings).
- Suboptimal allocations or unnecessary clones of JSON Values in hot execution paths.
- Accessibility gaps in Atelier UI (e.g., missing `aria-label` on icon-only buttons).
- Missing docstrings on public APIs or non-obvious procedural macro AST parsing steps.
- Inconsistent naming conventions across crate boundaries.

---

## 🔵 NIT / OBSERVATION (P3 / Low)
**Criteria**: Stylistic enhancements, minor idiomatic conventions, or micro-optimizations that can be accepted or deferred at the author's discretion.

### Concrete Examples:
- Minor variable or function naming improvements for enhanced clarity.
- Using idiomatic language constructs (e.g., `if let Some(x)` vs `match`).
- Typo corrections in internal comments or docstrings.
- Ordering of import statements or structuring of helper modules.
