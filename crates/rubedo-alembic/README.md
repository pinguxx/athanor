# rubedo-alembic

In-process WebAssembly sandbox runtime using Wasmtime 25.0 and WASI for the Rubedo agentic framework.

## Features
- **Deterministic Instruction Fuel Metering**: Terminate runaway compute or infinite loops in $<15\text{ ms}$.
- **Linear Memory Envelopes**: Configurable guest allocation caps (default 64MB).
- **Zero-Access WASI**: Sandbox isolation forbidding unauthorized filesystem and network access.
- **Dynamic Reagent Synthesis**: Transparently wrap WASM capabilities as native Rubedo Reagents.
