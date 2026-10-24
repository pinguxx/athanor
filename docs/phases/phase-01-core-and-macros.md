# Phase 1: Foundation & `#[reagent]` Macro Synthesizer

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-core`, `crates/rubedo-macros`
- **Primary Goal**: Establish core trait contracts, the unified error hierarchy, agent execution context, and the procedural macro (`#[reagent]`) that synthesizes JSON Schema directly from async Rust functions.

---

## 2. Detailed Task Breakdown

### Task 1.1: Cargo Workspace Initialization
- Create root `Cargo.toml` with workspace members:
  ```toml
  [workspace]
  members = [
      "crates/rubedo-core",
      "crates/rubedo-macros",
      "crates/rubedo-opus",
      "crates/rubedo-crucible",
      "crates/rubedo-alembic",
      "crates/rubedo-mcp",
      "crates/rubedo-rag",
      "crates/rubedo-evals",
      "crates/rubedo-atelier",
  ]
  resolver = "2"
  ```
- Declare shared workspace dependencies: `tokio`, `serde`, `serde_json`, `schemars`, `async-trait`, `thiserror`, `tracing`, `tracing-subscriber`, `syn`, `quote`, `proc-macro2`.

### Task 1.2: Scaffold `rubedo-core`
- Create `crates/rubedo-core/Cargo.toml` inheriting workspace dependencies (`serde`, `serde_json`, `schemars`, `async-trait`, `thiserror`, `tokio`).
- Define `crates/rubedo-core/src/lib.rs` exporting:
  - `pub mod reagent;`
  - `pub mod artificer;`
  - `pub mod context;`
  - `pub mod envelope;`
  - `pub mod error;`

### Task 1.3: Implement `ReagentError` and `Reagent` Trait
- In `crates/rubedo-core/src/reagent.rs`:
  ```rust
  use async_trait::async_trait;
  use serde_json::Value;

  #[derive(Debug, thiserror::Error)]
  pub enum ReagentError {
      #[error("Validation failed: {0}")]
      InvalidInput(String),

      #[error("Recoverable error: {message}. Guidance: {guidance}")]
      Recoverable {
          message: String,
          guidance: String,
          suggested_fix: Option<Value>,
      },

      #[error("Fatal reagent failure: {0}")]
      Fatal(String),
  }

  #[async_trait]
  pub trait Reagent: Send + Sync + 'static {
      fn name(&self) -> &'static str;
      fn description(&self) -> &'static str;
      fn input_schema(&self) -> Value;
      fn output_schema(&self) -> Value;
      async fn transmute(&self, input: Value) -> Result<Value, ReagentError>;
  }
  ```

### Task 1.4: Implement `Artificer` & `AthanorContext`
- In `crates/rubedo-core/src/artificer.rs`:
  ```rust
  use std::sync::Arc;
  use crate::reagent::Reagent;

  pub struct Artificer {
      pub name: String,
      pub instructions: String,
      pub model: String,
      pub reagents: Vec<Arc<dyn Reagent>>,
      pub max_steps: usize,
  }
  ```
- In `crates/rubedo-core/src/context.rs`:
  - Implement `AthanorContext` maintaining conversation message history, token metrics, and execution cancellation tokens.
- In `crates/rubedo-core/src/envelope.rs`:
  - Implement `Envelope<T>` value wrapper satisfying `'static + Send + Sync`.

### Task 1.5: Scaffold `rubedo-macros`
- Create `crates/rubedo-macros/Cargo.toml` with `proc-macro = true`:
  - Dependencies: `syn`, `quote`, `proc-macro2`, `schemars`, `serde`.

### Task 1.6: Implement `#[reagent]` Procedural Macro
- In `crates/rubedo-macros/src/lib.rs`:
  1. Parse macro arguments (`#[reagent(name = "...")]`).
  2. Parse target async function using `syn::ItemFn`.
  3. Extract doc comments as tool description.
  4. Generate input struct named `{FunctionName}Input` containing function parameters, with derived `serde::Deserialize` and `schemars::JsonSchema`.
  5. Generate struct `{FunctionName}Reagent`.
  6. Implement `Reagent` for `{FunctionName}Reagent`:
     - `name()` returns specified or function name.
     - `description()` returns parsed doc comments.
     - `input_schema()` generates schema via `schemars::schema_for!({FunctionName}Input)`.
     - `output_schema()` generates schema for return type.
     - `transmute()` deserializes `serde_json::Value` into `{FunctionName}Input` and awaits the original function.

---

## 3. Verification & Acceptance Criteria

### Unit & Integration Test
Create `crates/rubedo-macros/tests/reagent_test.rs`:
```rust
use rubedo_core::reagent::{Reagent, ReagentError};
use rubedo_macros::reagent;
use serde_json::json;

/// Search query tool for testing.
#[reagent(name = "test_tool")]
async fn test_tool(
    /// The search term to find.
    query: String
) -> Result<String, ReagentError> {
    Ok(format!("Result: {}", query))
}

#[tokio::test]
async fn test_reagent_schema_and_transmute() {
    let tool = TestToolReagent;
    assert_eq!(tool.name(), "test_tool");
    assert_eq!(tool.description().trim(), "Search query tool for testing.");

    let schema = tool.input_schema();
    assert!(schema.to_string().contains("query"));

    let input = json!({ "query": "alchemy" });
    let output = tool.transmute(input).await.expect("transmute failed");
    assert_eq!(output, json!("Result: alchemy"));
}
```

### Execution Command
```bash
cargo test -p rubedo-core -p rubedo-macros
```
