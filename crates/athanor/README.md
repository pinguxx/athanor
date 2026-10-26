# athanor

[![Crates.io](https://img.shields.io/crates/v/athanor.svg)](https://crates.io/crates/athanor)
[![Documentation](https://docs.rs/athanor/badge.svg)](https://docs.rs/athanor)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/pinguxx/athanor)

Top-level umbrella crate for the **Athanor** agentic framework in Rust.

## Features

Activate only the subsystems your agent application requires:

```toml
[dependencies]
# Full framework with all capabilities
athanor = { version = "0.1", features = ["full"] }

# Or cherry-pick individual subsystems:
# athanor = { version = "0.1", features = ["opus", "mcp"] }
```

| Feature | Subsystem | Description |
|:---|:---|:---|
| `macros` | `athanor-macros` | `#[reagent]` compile-time JSON Schema generation (default) |
| `crucible` | `athanor-crucible` | SQLite WAL event sourcing & transactional checkpoints |
| `opus` | `athanor-opus` | Durable DAG workflow engine with tokenized pause/resume |
| `alembic` | `athanor-alembic` | In-process WebAssembly sandbox with fuel limits |
| `mcp` | `athanor-mcp` | Bidirectional Model Context Protocol (MCP) host & client |
| `rag` | `athanor-rag` | In-process vector store via FastEmbed & LanceDB |
| `atelier` | `athanor-atelier` | Embedded Dev Studio visualizer & SSE telemetry |
| `evals` | `athanor-evals` | Benchmark suites, token guardrails, and LLM judges |
| `full` | All Subsystems | Enables all features above |

## Quickstart

```rust
use athanor::prelude::*;

#[reagent(name = "calculator")]
async fn calculate(operation: String, a: f64, b: f64) -> Result<f64, ReagentError> {
    match operation.as_str() {
        "add" => Ok(a + b),
        "multiply" => Ok(a * b),
        _ => Err(ReagentError::invalid_input("Unsupported operation")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let artificer = Artificer::builder()
        .name("Hermes")
        .instructions("Analytical agent assistant.")
        .model("claude-3-5-sonnet")
        .reagent(std::sync::Arc::new(CalculateReagent))
        .build()?;

    println!("Artificer initialized with tools: {:?}", artificer.reagent_names());
    Ok(())
}
```
