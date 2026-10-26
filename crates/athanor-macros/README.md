# athanor-macros

Procedural macros synthesizing JSON Schemas and executable `Reagent` capabilities for the **Athanor** agentic framework.

## Overview

The `#[reagent]` attribute macro transforms standard asynchronous Rust functions into schema-validated, thread-safe `Reagent` instances:

1. Synthesizes an input parameter struct deriving `serde::Deserialize`, `serde::Serialize`, and `schemars::JsonSchema`.
2. Emits doc comments into field descriptions within the emitted JSON Schema.
3. Implements the `athanor_core::reagent::Reagent` trait with automatic argument deserialization, validation, execution, and error mapping.
4. Preserves the original async function for direct invocation.
