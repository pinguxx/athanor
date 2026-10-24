# Phase 8: Reference Examples & Release Hardening

## 1. Objectives & Overview
- **Target Directories**: `examples/*`, root workspace verification
- **Primary Goal**: Construct canonical reference applications demonstrating each major feature of Rubedo. Verify full workspace build integrity, run end-to-end integration tests, enforce linting standards, and establish the v0.1.0 release packaging.

---

## 2. Detailed Task Breakdown

### Task 8.1: Implement `basic-artificer` Example
- Path: `examples/basic-artificer/`
- Contents:
  - Demonstrates a ReAct agent using `#[reagent]` tools (e.g. calculator, weather or web fetch mock).
  - Initializes `Artificer` with instructions, model configuration, and reagents.
  - Executes multi-step reasoning loop and outputs verified final result.

### Task 8.2: Implement `durable-transmutation` Example
- Path: `examples/durable-transmutation/`
- Contents:
  - Demonstrates long-running workflow with human-in-the-loop approval.
  - Step 1: Drafts proposal and commits state.
  - Step 2: Suspends execution and prints resume token to console.
  - External trigger simulates human approval, resumes via `OpusEngine::resume()`.
  - Step 3: Completes transaction and verifies SQLite event audit trail.

### Task 8.3: Implement `sandboxed-alembic` Example
- Path: `examples/sandboxed-alembic/`
- Contents:
  - Demonstrates compiling a dynamic guest function into WASM.
  - Loads and runs the module inside `AlembicSandbox`.
  - Demonstrates strict fuel limiting aborting an infinite loop without crashing the process.

### Task 8.4: Workspace Verification & Quality Gates
- Execute `cargo clippy --workspace --all-targets -- -D warnings`
- Verify zero `unwrap()` or `expect()` in non-test crates.
- Check that all workspace crates build cleanly: `cargo build --workspace --all-targets`
- Run full workspace test suite: `cargo test --workspace`
- Verify documentation generation: `cargo doc --workspace --no-deps`

---

## 3. Verification & Acceptance Criteria

### Execution Commands
```bash
# Build all crates and examples
cargo build --workspace --examples

# Run complete workspace test matrix
cargo test --workspace

# Check clippy warnings
cargo clippy --workspace --all-targets -- -D warnings
```
