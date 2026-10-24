# RUBEDO: Ironclad Agentic Framework for Rust

> **Rubedo (*Redness*)**: The fourth and final stage of the alchemical *Magnum Opus*, transmuting raw base matter into gold. In this framework, chaotic, non-deterministic agent inputs and LLM streams are distilled into deterministic, verified, and durable state transitions using Rust’s compile-time safety and async runtime.

---

## 🏛 Architectural Tenets

1. **Zero Lifetime Annotations in Macros**: Public macros (`#[reagent]`, `opus!`, `artificer!`) mandate `'static + Send + Sync` via value semantics and `Arc` envelopes. Autonomous authoring agents can generate valid code without lifetime borrow errors.
2. **Deterministic Checkpointing**: Workflows commit state before and after each Vessel execution. Any node can suspend execution (e.g., human-in-the-loop, external webhook) and resume seamlessly from an unforgeable token.
3. **In-Process Containment**: Dynamic and third-party tools run inside `wasmtime` sandboxes with strict memory envelopes (64 MB) and instruction fuel tracking (halting infinite loops in $<15$ ms).
4. **Bidirectional Model Context Protocol (MCP)**: Every Reagent and Artificer automatically exposes an MCP server interface while seamlessly consuming remote MCP endpoints.
5. **Zero External Infrastructure Required**: Embedded SQLite (`sqlx`) and embedded vector search (`fastembed-rs` + `lancedb`) run in-process within a sub-40MB memory footprint.

---

## ⚗ Alchemical Lexicon

| Alchemical Term | Framework Component | Role & Responsibility |
|:---|:---|:---|
| **Rubedo** | Top-level runtime | Root framework orchestrator, builder, and execution coordinator. |
| **Artificer** | Agent (`Agent`) | Autonomous reasoning loop executing over conversation history, context, and reagents. |
| **Reagent** | Tool (`Tool`) | Executable capability with compile-time generated JSON Schema and runtime validation. |
| **Opus** | Workflow (`Workflow`) | Directed acyclic graph (DAG) of steps with event-sourced persistence and checkpointing. |
| **Vessel** | Step (`Step`) | Individual execution node within an Opus capable of state transformation or suspension. |
| **Crucible** | State Store (`State`) | SQLite-backed event store enabling pause, resume, rollback, and time-travel replay. |
| **Alembic** | Sandbox (`Sandbox`) | In-process, fuel-metered WASM runtime (`wasmtime`) for untrusted tools. |
| **Atelier** | Dev Studio (`Studio`) | Embedded Axum HTTP/SSE server + single-binary Svelte visualizer for live telemetry. |

---

## 📦 Workspace & Crate Layout

```text
rubedo/
├── Cargo.toml                       # Workspace definition
├── crates/
│   ├── rubedo-core/                 # Core traits: Reagent, Artificer, Envelope, Error
│   ├── rubedo-macros/               # Proc-macros: #[reagent], opus!, artificer!
│   ├── rubedo-opus/                 # Durable DAG workflow engine
│   ├── rubedo-crucible/             # SQLite state persistence & event sourcing
│   ├── rubedo-alembic/              # Sandboxed WASM execution via Wasmtime
│   ├── rubedo-mcp/                  # Bidirectional MCP host/client implementation
│   ├── rubedo-rag/                  # In-process embeddings & vector storage
│   ├── rubedo-evals/                # Evaluator harness, assertions, and token guardrails
│   └── rubedo-atelier/              # Axum server + rust-embed Svelte dev visualizer
└── examples/
    ├── basic-artificer/             # ReAct agent with custom reagents
    ├── durable-transmutation/       # Suspended workflow with tokenized resumption
    └── sandboxed-alembic/           # Executing untrusted WASM code under fuel limits
```

---

## 📚 Documentation Index

We have structured comprehensive product, architectural, and phased implementation plans to guide the step-by-step construction of the framework:

- **[Product Plan & Strategic Roadmap](file:///c:/dev/Rubedo/PRODUCT_PLAN.md)**: Market opportunity, target personas, competitive landscape, non-functional performance budgets, and product milestones.
- **[Technical Specification](file:///c:/dev/Rubedo/RUBEDO_SPEC.md)**: Complete contracts for all traits, structs, error types, SQLite schemas, WASM isolation, MCP protocols, and REST/SSE endpoints.
- **[Subsystem Architecture](file:///c:/dev/Rubedo/docs/ARCHITECTURE.md)**: Deep dive into the "Zero Lifetime Annotations" pattern, event-sourced state machines, WASM fuel limits, and in-process RAG.
- **[Master Implementation Plan](file:///c:/dev/Rubedo/IMPLEMENTATION_PLAN.md)**: Master phase-by-phase checklist with verifiable milestones and exit criteria.
- **[Agent Execution Guide](file:///c:/dev/Rubedo/docs/AGENT_GUIDE.md)**: Ready-to-use prompt templates for Antigravity or AI coding assistants to execute each phase sequentially.

### Phased Execution Guides

1. **[Phase 1: Foundation & `#[reagent]` Macro Synthesizer](file:///c:/dev/Rubedo/docs/phases/phase-01-core-and-macros.md)**
2. **[Phase 2: Durable Opus & Crucible Persistence](file:///c:/dev/Rubedo/docs/phases/phase-02-opus-and-crucible.md)**
3. **[Phase 3: Alembic WASM Sandbox Runtime](file:///c:/dev/Rubedo/docs/phases/phase-03-alembic-wasm.md)**
4. **[Phase 4: Bidirectional Model Context Protocol (MCP) Gateway](file:///c:/dev/Rubedo/docs/phases/phase-04-mcp-gateway.md)**
5. **[Phase 5: Spagyric RAG & In-Process Vector Store](file:///c:/dev/Rubedo/docs/phases/phase-05-spagyric-rag.md)**
6. **[Phase 6: Atelier Dev Studio & Observability Server](file:///c:/dev/Rubedo/docs/phases/phase-06-atelier-studio.md)**
7. **[Phase 7: Verification Harness & Evaluation Guardrails](file:///c:/dev/Rubedo/docs/phases/phase-07-evals-and-guardrails.md)**
8. **[Phase 8: Reference Examples & Release Hardening](file:///c:/dev/Rubedo/docs/phases/phase-08-examples-and-release.md)**

---

## ⚡ Quickstart Preview: Declaring a Reagent

```rust
use rubedo_macros::reagent;
use rubedo_core::reagent::ReagentError;

/// Fetches the current gold-to-silver exchange ratio.
#[reagent(name = "get_transmutation_rate")]
async fn get_transmutation_rate(
    /// Base metal name (e.g., "lead", "silver")
    base_metal: String,
) -> Result<f64, ReagentError> {
    match base_metal.to_lowercase().as_str() {
        "silver" => Ok(0.0125),
        "lead" => Ok(0.00005),
        _ => Err(ReagentError::Recoverable {
            message: "Unsupported base metal".into(),
            guidance: "Specify either 'silver' or 'lead'".into(),
            suggested_fix: None,
        }),
    }
}
```

---

## 🛡 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
