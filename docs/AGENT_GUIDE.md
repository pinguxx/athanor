# Antigravity Agent Execution Guide & Prompting Handbook

This handbook provides standard operating procedures, strict implementation constraints, and copy-paste prompt templates for executing each phase of **Rubedo** using Antigravity or related AI coding agents.

---

## 1. Golden Rules of Engagement

1. **Zero Lifetime Annotations**: All public types passing through `Opus`, `Reagent`, and `Artificer` must be `'static + Send + Sync`. Never generate code requiring lifetime parameters (`'a`).
2. **Error Handling Discipline**: Always use `thiserror` for error definitions. **Never** use `.unwrap()` or `.expect()` in non-test crate code. Return structured `Result<T, E>` at all times.
3. **Workspace Dependency Alignment**: Never add random dependencies or drift crate versions. Inherit or pin versions strictly aligned with root `Cargo.toml`.
4. **Async Trait Standard**: All asynchronous trait declarations must be annotated with `#[async_trait]`.
5. **Test-First Verification**: Every phase deliverable must include unit tests and an integration test under `crates/[CRATE_NAME]/tests/`. A phase is only complete when `cargo test -p [CRATE_NAME]` succeeds with 0 errors.

---

## 2. Universal Agent Execution Prompt Template

When invoking an agent for any phase, use this structured prompt:

```markdown
You are implementing Phase [NUMBER]: [NAME] for the Rubedo agent framework.

Context:
Review the subsystem specifications in RUBEDO_SPEC.md and the detailed phase guide in docs/phases/phase-[NUMBER]-[SLUG].md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Create and populate the crate folder: crates/[CRATE_NAME]
- Implement all required data structures and trait contracts.
- Write inline unit tests and add an integration test under crates/[CRATE_NAME]/tests/.
- Run `cargo check` and `cargo test -p [CRATE_NAME]` to verify the implementation builds cleanly and passes all tests.
```

---

## 3. Ready-to-Use Phase Invocations

### Phase 1 Prompt
```markdown
You are implementing Phase 1: Foundation & #[reagent] Macro Synthesizer for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-01-core-and-macros.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Initialize root Cargo.toml workspace.
- Create and populate crates/rubedo-core (Reagent, ReagentError, Artificer, AthanorContext).
- Create and populate crates/rubedo-macros (proc macro #[reagent] generating JSON schemas via schemars).
- Add integration test under crates/rubedo-macros/tests/reagent_test.rs.
- Verify with `cargo test -p rubedo-core -p rubedo-macros`.
```

### Phase 2 Prompt
```markdown
You are implementing Phase 2: Durable Opus & Crucible Persistence for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-02-opus-and-crucible.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-crucible with sqlx SQLite migrations (opus_runs, vessel_events, suspended_tokens).
- Implement CrucibleEngine for state persistence, transition logging, and checkpoints.
- Scaffold crates/rubedo-opus with Vessel trait, OpusBuilder, and OpusEngine state machine.
- Implement tokenized suspension and OpusEngine::resume().
- Add integration test under crates/rubedo-opus/tests/durable_workflow_test.rs.
- Verify with `cargo test -p rubedo-crucible -p rubedo-opus`.
```

### Phase 3 Prompt
```markdown
You are implementing Phase 3: Alembic WASM Sandbox Runtime for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-03-alembic-wasm.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-alembic with wasmtime and wasmtime-wasi.
- Implement AlembicConfig (max_memory_bytes, fuel_limit) and AlembicSandbox.
- Build host-guest linear memory bridge and wrap WASM modules into rubedo_core::Reagent.
- Add integration test under crates/rubedo-alembic/tests/fuel_exhaustion_test.rs asserting infinite loop halts in <15ms.
- Verify with `cargo test -p rubedo-alembic`.
```

### Phase 4 Prompt
```markdown
You are implementing Phase 4: Bidirectional Model Context Protocol (MCP) Gateway for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-04-mcp-gateway.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-mcp with JSON-RPC 2.0 serialization.
- Implement McpServer exposing native Reagents over stdio/in-memory duplex.
- Implement McpClient discovering tools and wrapping them into dynamic Reagents.
- Add integration test under crates/rubedo-mcp/tests/bidirectional_mcp_test.rs.
- Verify with `cargo test -p rubedo-mcp`.
```

### Phase 5 Prompt
```markdown
You are implementing Phase 5: Spagyric RAG & In-Process Vector Store for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-05-spagyric-rag.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-rag with fastembed (BGESmallENV15) and lancedb.
- Implement text chunkers and SpagyricIndex (add_documents, search).
- Implement dynamic SearchReagent for Artificers.
- Add integration test under crates/rubedo-rag/tests/semantic_search_test.rs verifying cosine similarity >0.70.
- Verify with `cargo test -p rubedo-rag`.
```

### Phase 6 Prompt
```markdown
You are implementing Phase 6: Atelier Dev Studio & Observability Server for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-06-atelier-studio.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-atelier with axum, tower-http, and rust-embed.
- Configure custom tracing layer streaming to Tokio broadcast channel.
- Implement HTTP routes: /api/registry, /api/events (SSE), /api/opus/:run_id/resume, and embedded static asset fallback.
- Add integration test under crates/rubedo-atelier/tests/atelier_server_test.rs.
- Verify with `cargo test -p rubedo-atelier`.
```

### Phase 7 Prompt
```markdown
You are implementing Phase 7: Verification Harness & Evaluation Guardrails for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-07-evals-and-guardrails.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Scaffold crates/rubedo-evals.
- Implement ExactMatch, LevenshteinDistance, and JsonGrammarValidator.
- Implement LlmJudge and TokenBudgetGuard.
- Implement #[artificer_test] procedural macro for running benchmarks.
- Add integration test under crates/rubedo-evals/tests/eval_harness_test.rs.
- Verify with `cargo test -p rubedo-evals`.
```

### Phase 8 Prompt
```markdown
You are implementing Phase 8: Reference Examples & Release Hardening for the Rubedo agent framework.

Context:
Review RUBEDO_SPEC.md and docs/phases/phase-08-examples-and-release.md.

Implementation Constraints:
1. Ensure all types passing through Opus, Reagent, and Artificer are 'static + Send + Sync.
2. Use thiserror for all error declarations. Explicitly forbid unwrap() and expect() in non-test code paths.
3. Keep external dependencies aligned with the root Cargo.toml workspace definitions.
4. Ensure all asynchronous traits use #[async_trait].

Deliverables:
- Implement examples/basic-artificer.
- Implement examples/durable-transmutation.
- Implement examples/sandboxed-alembic.
- Verify `cargo clippy --workspace --all-targets -- -D warnings`.
- Verify `cargo test --workspace`.
```
