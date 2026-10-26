# Contributing to Athanor

Thank you for your interest in contributing to **Athanor**! We welcome contributions that uphold the framework's commitment to reliability, memory safety, and deterministic agent execution.

---

## 🏛 Core Architectural Invariants

Every contribution must strictly adhere to the framework's foundational tenets:

1. **Zero Lifetime Annotations**:
   - All types passing through `Opus`, `Reagent`, and `Artificer` must be `'static + Send + Sync`.
   - Macros and public traits must rely on owned value semantics and `Arc` envelopes. Never introduce public lifetime parameters (`'a`).
2. **Ironclad Error Handling**:
   - Use `thiserror` for internal and public error hierarchies.
   - **Never call `.unwrap()` or `.expect()`** in non-test crate sources. Return structured `Result<T, E>` with actionable remediation guidance.
3. **100% Safe Rust**:
   - `#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]` is enforced across all crates. Any necessary `unsafe` code must be isolated and documented with explicit `// SAFETY` justification.
4. **Asynchronous Trait Standard**:
   - All asynchronous trait definitions must be annotated with `#[async_trait]`.
5. **Deterministic Containment**:
   - Third-party code execution must be sandboxed inside WebAssembly (`athanor-alembic`) under strict memory ceilings and instruction fuel meters.

---

## 🛠 Development Workflow

### Prerequisites
- **Rust Toolchain**: 1.80+ (`rustup default stable`)
- **Protobuf Compiler**: `protoc` (required by LanceDB / vector dependencies)

### Building the Workspace
```bash
cargo build --workspace
```

### Running Tests
All contributions must include thorough unit and integration tests:
```bash
cargo test --workspace
```

### Code Formatting & Linting
Ensure formatting and clippy checks pass with zero warnings:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

### Automated Master Audit Pipeline
Before opening a pull request, run the master audit verification suite:
```powershell
powershell -ExecutionPolicy Bypass -File .agents/scripts/run_post_task_audits.ps1
```

---

## 📝 Pull Request Guidelines

1. **Focused Changes**: Keep pull requests focused on a single feature, bug fix, or subsystem improvement.
2. **Test Coverage**: Maintain $>80\%$ line coverage on new code. Add integration tests under `crates/[crate_name]/tests/`.
3. **Commit Messages**: Follow standard conventional commits format (`feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`).
4. **Documentation**: Update crate-level `README.md` and inline docstrings for any added or modified public symbols.
