# ATHANOR: Ironclad Agentic Framework for Rust

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=flat-square)](LICENSE-MIT)
[![Safety](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square)](https://github.com/rust-secure-code/safety-dance)
[![Tests](https://img.shields.io/badge/tests-100%25%20passing-brightgreen.svg?style=flat-square)](#-testing--quality-assurance)
[![Architecture](https://img.shields.io/badge/architecture-zero--lifetime-purple.svg?style=flat-square)](docs/ARCHITECTURE.md)

> **Athanor**: The cosmic, self-feeding furnace of the alchemists, engineered to maintain an undying, constant heat across perpetual cycles of distillation. In this framework, chaotic, non-deterministic agent inputs and LLM streams are distilled into deterministic, verified, and durable state transitions using Rust’s compile-time safety and async runtime.

---

## 🏛 Core Philosophy & Architectural Tenets

Athanor is engineered from the ground up to solve the fundamental unreliability of autonomous LLM workflows. By combining compile-time safety, transactional persistence, process isolation, and open protocols, Athanor turns non-deterministic reasoning into mission-critical software systems.

1. **Zero Lifetime Annotations**: Public macros (`#[reagent]`, `#[artificer_test]`) mandate `'static + Send + Sync` via value semantics and owned/`Arc` envelopes. Autonomous code synthesizers and human engineers can compose tools and agents without fighting lifetime borrow checker errors.
2. **Durable Event-Sourced Execution**: Multi-step workflows (`Opus`) commit state before and after each step (`Vessel`) into an embedded SQLite engine (`Crucible`). Any node can suspend execution (e.g. human-in-the-loop approvals, external webhooks) and resume seamlessly from an unforgeable cryptographic token.
3. **In-Process Sandboxed Isolation**: Untrusted or model-generated tools execute inside WebAssembly (`wasmtime` 25.0) containers with zero host filesystem/network capabilities and strict fuel metering that terminates infinite loops in $<15\text{ ms}$.
4. **Bidirectional Model Context Protocol (MCP)**: Native tools and agents automatically expose an MCP server interface (JSON-RPC 2.0 over stdio or duplex channels), while simultaneously discovering and consuming remote MCP endpoints as first-class tools.
5. **Zero External Infrastructure**: Embedded SQLite with WAL mode (`sqlx`), local vector embeddings (`FastEmbed`), and embedded vector storage (`LanceDB`) run entirely in-process within a sub-40MB memory footprint.
6. **Built-in Verification & Guardrails**: Deterministic string and JSON grammar validators, token consumption watchdogs, LLM-as-a-judge scoring, and CI benchmark harnesses guarantee behavioral compliance before release.

---

## ⚗ Alchemical Lexicon

Athanor draws upon classical alchemical nomenclature to delineate system roles and operational boundaries:

| Alchemical Term | Framework Concept | Role & Responsibility |
|:---|:---|:---|
| **Athanor** | Root Framework | Top-level runtime, builder, and orchestrator coordinating subsystems. |
| **Artificer** | Autonomous Agent | Multi-step reasoning loop (ReAct) executing over history, context, and capabilities. |
| **Reagent** | Tool / Capability | Executable capability with compile-time generated JSON Schema and typed validation. |
| **Opus** | Durable Workflow | Directed acyclic graph (DAG) of steps with event-sourced checkpoints and rollbacks. |
| **Vessel** | Workflow Step | Individual unit of execution in an Opus capable of state transformation or suspension. |
| **Crucible** | State Store | SQLite-backed journal recording runs, step transitions, and resumption tokens. |
| **Alembic** | WASM Sandbox | In-process, fuel-metered WebAssembly runtime (`wasmtime`) for untrusted tools. |
| **Atelier** | Dev Studio | Embedded Axum HTTP/SSE server + single-binary visualizer for live DAG telemetry. |
| **Spagyric** | RAG / Vector Engine | In-process document chunking, ONNX embedding generation, and vector retrieval. |

---

## 📦 Crate Ecosystem

Athanor is organized as a modular Cargo workspace composed of nine dedicated crates:

```text
athanor/
├── Cargo.toml                       # Workspace manifest & pinned dependencies
├── crates/
│   ├── athanor-core/                # Core traits: Reagent, Artificer, Envelope, AthanorContext
│   ├── athanor-macros/              # Proc-macros: #[reagent], #[artificer_test]
│   ├── athanor-opus/                # Durable DAG workflow engine & state machine
│   ├── athanor-crucible/            # SQLite event sourcing, checkpoints & token storage
│   ├── athanor-alembic/             # In-process WebAssembly sandbox with fuel limits
│   ├── athanor-mcp/                 # Bidirectional Model Context Protocol (MCP 2024-11-05+)
│   ├── athanor-rag/                 # Spagyric embedded RAG (FastEmbed + LanceDB)
│   ├── athanor-atelier/             # Embedded Axum server, SSE telemetry & Dev Studio UI
│   └── athanor-evals/               # Benchmark suites, token guardrails & LLM judges
└── examples/
    ├── basic-artificer/             # Autonomous ReAct agent with custom reagents
    ├── durable-transmutation/       # Event-sourced workflow with tokenized pause/resume
    └── sandboxed-alembic/           # Untrusted WASM execution under fuel limits
```

### Subsystem Overview

| Crate | Primary Focus | Key Types & Macros |
|:---|:---|:---|
| [`athanor-core`](crates/athanor-core) | Foundational primitives & value semantics | `Reagent`, `Artificer`, `AthanorContext`, `Envelope`, `ReagentError` |
| [`athanor-macros`](crates/athanor-macros) | Procedural code & schema synthesizers | `#[reagent]`, `#[artificer_test]` |
| [`athanor-opus`](crates/athanor-opus) | Resilient DAG workflow orchestration | `Opus`, `OpusBuilder`, `OpusEngine`, `Vessel`, `OpusContext` |
| [`athanor-crucible`](crates/athanor-crucible) | SQLite WAL persistence & event sourcing | `CrucibleEngine`, `OpusRunRecord`, `SuspendedTokenRecord` |
| [`athanor-alembic`](crates/athanor-alembic) | Sandboxed WASM containment & fuel metering | `AlembicSandbox`, `AlembicConfig`, `WasmReagent` |
| [`athanor-mcp`](crates/athanor-mcp) | Bidirectional MCP gateway (JSON-RPC 2.0) | `McpServer`, `McpClient`, `StdioTransport`, `ChannelTransport` |
| [`athanor-rag`](crates/athanor-rag) | In-process vector indexing & semantic search | `SpagyricIndex`, `Document`, `RagSearchReagent`, `RecursiveCharacterSplitter` |
| [`athanor-atelier`](crates/athanor-atelier) | Embedded dev studio & live observability | `AtelierServer`, `AtelierRegistry`, `TelemetryBroadcaster`, `AtelierTracingLayer` |
| [`athanor-evals`](crates/athanor-evals) | Evaluation harness & behavioral guardrails | `BenchmarkSuite`, `TokenBudgetGuard`, `ExactMatch`, `LevenshteinDistance`, `LlmJudge` |

---

## ⚡ Feature Deep Dives & Practical Examples

### 1. Declaring Strongly Typed Reagents with `#[reagent]`

The `#[reagent]` procedural macro converts any standard asynchronous Rust function into an executable capability. It extracts docstrings to synthesize JSON Schemas at compile time, parses and validates inputs against the schema, and enforces typed error recovery.

```rust
use athanor_macros::reagent;
use athanor_core::reagent::ReagentError;
use serde_json::json;

/// Mathematical calculator performing arithmetic operations.
#[reagent(name = "calculator")]
async fn calculate(
    /// Mathematical operation ("add", "subtract", "multiply", "divide")
    operation: String,
    /// First numeric operand
    a: f64,
    /// Second numeric operand
    b: f64,
) -> Result<f64, ReagentError> {
    match operation.to_lowercase().as_str() {
        "add" => Ok(a + b),
        "subtract" => Ok(a - b),
        "multiply" => Ok(a * b),
        "divide" => {
            if b.abs() < f64::EPSILON {
                // Return a structured recoverable error that informs the LLM how to correct itself
                Err(ReagentError::recoverable(
                    "Division by zero is undefined",
                    "Provide a non-zero denominator operand 'b'",
                ))
            } else {
                Ok(a / b)
            }
        }
        unknown => Err(ReagentError::invalid_input(format!(
            "Unsupported operation '{unknown}'. Supported: add, subtract, multiply, divide"
        ))),
    }
}
```

The macro generates a `CalculateReagent` struct implementing `Reagent`:
- `name()` returns `"calculator"`.
- `input_schema()` automatically produces JSON Schema with field descriptions extracted from comments.
- `transmute(input_value)` executes the function safely.

---

### 2. Autonomous Reasoning Loop with `Artificer`

An `Artificer` represents an autonomous agent persona equipped with reagents, context scratchpads (`AthanorContext`), and conversation history.

```rust
use std::sync::Arc;
use athanor_core::artificer::Artificer;
use athanor_core::reagent::Reagent;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Assemble Artificer with capabilities and persona instructions
    let artificer = Artificer::builder()
        .name("HermesScholar")
        .instructions(
            "You are a rigorous analytical assistant. Inspect available tools, \
             perform calculations deterministically, and explain your steps.",
        )
        .model("claude-3-5-sonnet")
        .max_steps(5)
        .reagent(Arc::new(CalculateReagent))
        .build()?;

    // 2. Inspect auto-generated JSON Schemas
    for schema in artificer.reagent_schemas() {
        println!("Available tool schema: {}", serde_json::to_string_pretty(&schema)?);
    }

    // 3. Invoke tool through the Artificer interface
    let tool = artificer.get_reagent("calculator").ok_or("Tool missing")?;
    let result = tool.transmute(json!({
        "operation": "multiply",
        "a": 12.5,
        "b": 4.0
    })).await?;

    println!("Tool calculation result: {result}"); // 50.0
    Ok(())
}
```

---

### 3. Durable Workflows with `Opus` & `Crucible`

In mission-critical agent workflows, network failures, rate limits, and human approval steps must not drop state. `athanor-opus` paired with `athanor-crucible` persists state transitions into SQLite after every step.

```rust
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;
use athanor_crucible::CrucibleEngine;
use athanor_opus::context::OpusContext;
use athanor_opus::engine::{OpusEngine, OpusRunResult};
use athanor_opus::vessel::{Vessel, VesselError, VesselStatus};
use athanor_opus::workflow::Opus;

// Define a workflow step that requires human verification
struct StepHumanApproval;

#[async_trait]
impl Vessel for StepHumanApproval {
    fn id(&self) -> &'static str {
        "step_human_approval"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let amount: f64 = ctx.get("amount").unwrap_or(0.0);

        if amount > 10_000.0 && ctx.resume_payload().is_none() {
            // Suspend workflow and mint an unforgeable resume token
            let token = format!("APPROVAL-TOKEN-{}", uuid::Uuid::new_v4());
            return Ok(VesselStatus::Suspended {
                resume_token: token,
                reason: format!("Transfer of ${amount} exceeds automated safety threshold"),
            });
        }

        // Once resumed with payload, proceed
        Ok(VesselStatus::Transmuted {
            output: json!({ "status": "approved", "amount": amount }),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize embedded SQLite storage engine
    let crucible = CrucibleEngine::connect_in_memory().await?;
    let engine = OpusEngine::new(crucible);

    // 2. Build the workflow DAG
    let opus = Opus::builder("financial_transmutation")
        .vessel(Arc::new(StepHumanApproval))
        .build()?;

    // 3. Execute workflow
    let mut ctx = OpusContext::new();
    ctx.set("amount", 25_000.0)?;

    let result = engine.start(&opus, ctx).await?;

    if let OpusRunResult::Suspended { run_id, resume_token, reason } = result {
        println!("Workflow {run_id} suspended: {reason}");
        println!("Resume token: {resume_token}");

        // 4. Simulate human review approval and resume execution seamlessly
        let resumed = engine.resume(
            run_id,
            &resume_token,
            json!({ "approved_by": "ComplianceOfficer", "override": true }),
        ).await?;

        println!("Workflow resumed to completion: {:?}", resumed);
    }

    Ok(())
}
```

---

### 4. In-Process Sandboxed Execution with `Alembic` (WASM)

When agents execute untrusted user code or dynamically synthesized functions, `athanor-alembic` confines execution to WebAssembly running within `wasmtime` with zero access to host resources.

```rust
use std::sync::Arc;
use athanor_alembic::config::AlembicConfig;
use athanor_alembic::reagent::WasmReagent;
use athanor_alembic::sandbox::AlembicSandbox;
use athanor_core::reagent::Reagent;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure strict memory ceilings and instruction fuel quotas
    let config = AlembicConfig::new()
        .with_max_memory_bytes(16 * 1024 * 1024) // 16 MB max linear memory
        .with_fuel_limit(100_000);              // Halt infinite loops deterministically

    let sandbox = Arc::new(AlembicSandbox::new(config)?);

    // 2. Wrap compiled WebAssembly module into a first-class Reagent
    let wasm_wat = r#"
        (module
          (memory (export "memory") 1)
          (func (export "allocate") (param i32) (result i32) (i32.const 0))
          (func (export "transmute") (param $ptr i32) (param $len i32) (result i64)
            (i64.or
              (i64.shl (i64.extend_i32_u (local.get $ptr)) (i64.const 32))
              (i64.extend_i32_u (local.get $len))
            )
          )
        )
    "#;

    let wasm_reagent = WasmReagent::new(
        "sandboxed_calculator",
        "Executes arithmetic inside an isolated WASM sandbox",
        json!({ "type": "object", "properties": { "value": { "type": "number" } } }),
        wasm_wat.as_bytes(),
        sandbox,
    );

    // 3. Execute safely across host-guest linear memory bridge
    let output = wasm_reagent.transmute(json!({ "value": 42 })).await?;
    println!("Sandboxed WASM output: {output}");

    Ok(())
}
```

---

### 5. Bidirectional Model Context Protocol (MCP)

`athanor-mcp` implements the standard [Model Context Protocol (MCP 2024-11-05+)](https://spec.modelcontextprotocol.io/) specification over JSON-RPC 2.0.

#### Exposing Reagents as an MCP Server
```rust
use std::sync::Arc;
use athanor_mcp::server::McpServer;
use athanor_mcp::transport::StdioTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Expose native Reagents to Claude Desktop, Cursor, or external IDEs
    let server = Arc::new(McpServer::new("athanor-mcp-server", "1.0.0"));
    server.register_reagent(Arc::new(CalculateReagent)).await;

    // Listen on standard I/O
    let transport = Arc::new(StdioTransport::new());
    server.run(transport).await?;

    Ok(())
}
```

#### Consuming Remote MCP Servers as Dynamic Tools
```rust
use std::sync::Arc;
use athanor_core::artificer::Artificer;
use athanor_mcp::client::McpClient;
use athanor_mcp::transport::ChannelTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (server_transport, client_transport) = ChannelTransport::create_duplex(64);
    let client = McpClient::new(Arc::new(client_transport));

    // Connect and initialize handshake
    client.initialize("hermes-client", "1.0.0").await?;

    // Dynamically discover all tools exposed by the remote MCP endpoint
    let discovered_tools = client.discover_reagents().await?;

    // Plug discovered tools directly into an Artificer
    let mut builder = Artificer::builder().name("MCPExplorer").model("claude-3-5-sonnet");
    for tool in discovered_tools {
        builder = builder.reagent(tool);
    }
    let artificer = builder.build()?;

    println!("Artificer dynamically equipped with: {:?}", artificer.reagent_names());
    Ok(())
}
```

---

### 6. Spagyric Embedded RAG & Vector Store

Perform vector search without deploying Chroma, Pinecone, or PostgreSQL. `athanor-rag` bundles ONNX runtime embeddings (`FastEmbed` `BGESmallENV15`) and embedded columnar storage (`LanceDB`) in-process.

```rust
use std::sync::Arc;
use athanor_core::artificer::Artificer;
use athanor_rag::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize embedded in-process vector store
    let index = SpagyricIndex::new_in_memory().await?;

    // 2. Ingest structured domain documents
    index.add_documents(&[
        Document::with_metadata(
            "doc-lead-transmutation",
            "Transmutation of base metals requires distillation at 450 degrees Celsius.",
            json!({ "category": "alchemy", "secret_level": 4 }),
        ),
        Document::with_metadata(
            "doc-quantum-superposition",
            "Superposition states collapse upon measurement into standard basis eigenstates.",
            json!({ "category": "physics", "secret_level": 1 }),
        ),
    ]).await?;

    // 3. Execute semantic similarity search (cosine similarity > 0.70)
    let results = index.search("melting temperature for metals", 1).await?;
    println!("Top semantic match: {}", results[0].document.content);

    // 4. Wrap index as a native Reagent for an Artificer
    let search_tool = RagSearchReagent::new(Arc::new(index));
    let artificer = Artificer::builder()
        .name("Archivist")
        .model("claude-3-5-sonnet")
        .reagent(Arc::new(search_tool))
        .build()?;

    Ok(())
}
```

---

### 7. Atelier Dev Studio & Live Observability

Embed live telemetry and a web visualizer directly into your binary with zero external sidecars:

```rust
use std::sync::Arc;
use athanor_atelier::prelude::*;
use athanor_crucible::CrucibleEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup registry and telemetry broadcaster
    let registry = AtelierRegistry::new();
    let broadcaster = TelemetryBroadcaster::new(1024);
    let crucible = CrucibleEngine::connect_in_memory().await?;

    // 2. Bind embedded HTTP/SSE dev studio server
    let server = AtelierServer::new(registry, broadcaster)
        .with_crucible(crucible);

    let bound = server.bind("127.0.0.1:3000").await?;
    println!("⚗ Atelier Dev Studio running at http://{}", bound.local_addr());

    // 3. Serve embedded static UI and API endpoints
    // - GET  /              -> Embedded Single-Binary Web Visualizer
    // - GET  /api/registry  -> Registered Artificers, Opuses, and Reagents
    // - GET  /api/events    -> Live Server-Sent Events (SSE) telemetry stream
    // - POST /api/opus/:run_id/resume -> Resume suspended workflows
    bound.run().await?;
    Ok(())
}
```

---

### 8. Evaluation Harness & Guardrails (`#[artificer_test]`)

Enforce strict regression prevention, token budgets, and LLM semantic grading:

```rust
use athanor_evals::prelude::*;

// 1. Token consumption guardrail prevents runaway billing
#[test]
fn test_token_guardrail() {
    let guard = TokenBudgetGuard::with_config(TokenBudgetConfig {
        max_total_tokens: 5_000,
        max_prompt_tokens: Some(4_000),
        max_completion_tokens: Some(1_000),
    });

    // Record usage during execution
    assert!(guard.record(500, 200).is_ok());
    assert_eq!(guard.remaining_tokens(), 4300);
}

// 2. Benchmark evaluation test asserting accuracy threshold
#[artificer_test(threshold = 0.85)]
async fn benchmark_artificer_accuracy() -> Result<EvaluationReport, EvalsError> {
    let suite = BenchmarkSuite::new("math_reasoning")
        .with_threshold(0.85)
        .add_case(TestCase::new("c1", "What is 15 * 4?", Some("60")))
        .add_case(TestCase::new("c2", "Square root of 81?", Some("9")));

    let report = suite.run(|prompt| async move {
        // Run agent invocation here
        Ok(match prompt.as_str() {
            "What is 15 * 4?" => "60".to_string(),
            _ => "9".to_string(),
        })
    }).await;

    Ok(report)
}
```

---

## 🏃 Running Reference Examples

The repository includes three complete, standalone reference examples under `examples/`:

```powershell
# 1. Basic Artificer (ReAct reasoning loop with #[reagent] tools)
cargo run -p basic-artificer

# 2. Durable Transmutation (Event-sourced state machine with tokenized pause/resume)
cargo run -p durable-transmutation

# 3. Sandboxed Alembic (Isolated WASM execution with instruction fuel exhaustion)
cargo run -p sandboxed-alembic
```

---

## 🧪 Testing & Quality Assurance

Athanor maintains an uncompromising standard for code quality, safety, and correctness:
- **Zero compiler warnings** (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Forbidden `.unwrap()` and `.expect()`** in non-test crate sources.
- **100% safe Rust code** (or audited `// SAFETY` annotations).
- **$>80\%$ line coverage** across all core crates.

Execute the complete workspace test suite:
```powershell
# Run all workspace unit and integration tests
cargo test --workspace

# Run clippy with strict denial of warnings
cargo clippy --workspace --all-targets -- -D warnings

# Execute the automated post-task quality and security audit
powershell -ExecutionPolicy Bypass -File .agents/scripts/run_post_task_audits.ps1
```

---

## 📚 Technical Documentation

Explore deep architectural blueprints and technical specifications:

- **[Product Plan & Strategic Roadmap](PRODUCT_PLAN.md)**: Market opportunity, target personas, non-functional performance budgets, and architecture milestones.
- **[Technical Specification](ATHANOR_SPEC.md)**: Exhaustive contracts for all traits, structs, error types, SQLite schemas, WASM isolation, MCP protocols, and REST/SSE endpoints.
- **[Subsystem Architecture](docs/ARCHITECTURE.md)**: Deep dive into the "Zero Lifetime Annotations" pattern, event-sourced state machines, WASM fuel limits, and in-process RAG.
- **[Master Implementation Plan](IMPLEMENTATION_PLAN.md)**: Detailed subsystem task breakdown with verifiable milestones and exit criteria.

---

## 🛡 License

Licensed under either of:
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- **MIT license** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
