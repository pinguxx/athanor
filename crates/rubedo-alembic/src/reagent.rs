//! Dynamic Reagent wrapper enabling sandboxed WebAssembly execution within Rubedo agent pipelines.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use rubedo_core::reagent::{Reagent, ReagentError};

use crate::sandbox::AlembicSandbox;

/// A dynamic Rubedo [`Reagent`] synthesized from a compiled WebAssembly guest module.
#[derive(Clone)]
pub struct WasmReagent {
    name: &'static str,
    description: &'static str,
    input_schema: Value,
    output_schema: Value,
    wasm_binary: Vec<u8>,
    sandbox: Arc<AlembicSandbox>,
}

impl std::fmt::Debug for WasmReagent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmReagent")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .field("output_schema", &self.output_schema)
            .field("wasm_binary_bytes", &self.wasm_binary.len())
            .finish()
    }
}

impl WasmReagent {
    /// Construct a new WASM-backed Reagent tool.
    pub fn new(
        name: &'static str,
        description: &'static str,
        input_schema: Value,
        wasm_binary: impl Into<Vec<u8>>,
        sandbox: Arc<AlembicSandbox>,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            output_schema: serde_json::json!({ "type": "object" }),
            wasm_binary: wasm_binary.into(),
            sandbox,
        }
    }

    /// Set a custom output schema.
    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = schema;
        self
    }

    /// Access the underlying WASM bytecode.
    pub fn wasm_binary(&self) -> &[u8] {
        &self.wasm_binary
    }
}

#[async_trait]
impl Reagent for WasmReagent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    fn output_schema(&self) -> Value {
        self.output_schema.clone()
    }

    async fn transmute(&self, input: Value) -> Result<Value, ReagentError> {
        self.sandbox
            .execute_raw(&self.wasm_binary, input)
            .map_err(ReagentError::from)
    }
}
