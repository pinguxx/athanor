# Rubedo Workspace Rules & Mandatory Quality Gates

## 1. Mandatory Post-Code Task Audit Protocol
Whenever you finish implementing a code task, feature, crate, or phase deliverable:
You MUST execute the master post-task audit pipeline to guarantee ironclad quality, security, and zero regressions before concluding the task:

```powershell
powershell -ExecutionPolicy Bypass -File .agents/scripts/run_post_task_audits.ps1
```

This automated verification suite executes:
1. **Critical Code Review (`critical-code-reviewer`)**:
   - Inspects git diff and assesses technology domain blast radius.
   - Audits concurrency invariants, failure handling, and state rollback paths.
2. **Security & Invariant Audit (`security-auditor`)**:
   - Scans for hardcoded secrets or credentials.
   - Forbids any `unwrap()` or `expect()` calls in non-test crate sources.
   - Enforces 100% safe Rust code (or audited `// SAFETY` annotations).
   - Enforces zero compiler/clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`.
   - Executes full workspace test suite: `cargo test --workspace`.

---

## 2. Implementation Invariants
- **Value Semantics**: All types passing through `Opus`, `Reagent`, and `Artificer` must be `'static + Send + Sync`.
- **Error Handling**: Use `thiserror` for all internal and public error hierarchies.
- **Async Trait Standard**: All asynchronous trait interfaces must use `#[async_trait]`.
- **Test Coverage**: Maintain $>80\%$ line coverage verified via `cargo llvm-cov`. Keep tests segregated under `tests/` directories.
