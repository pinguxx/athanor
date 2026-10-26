---
name: alembic-wasm-specialist
description: >-
  Specialist in Wasmtime 25.0, WebAssembly Text format (.wat), host-guest linear memory exchange,
  fuel-metered execution, and zero-access WASI sandboxing for athanor-alembic.
  Use when designing or debugging WASM isolation, host memory bridges, or untrusted tool execution.
---

# Alembic WASM Specialist (Wasmtime Sandboxing Guide)

A specialized skill for engineering, verifying, and debugging in-process WebAssembly sandboxes using **Wasmtime 25.0** and **wasmtime-wasi**. Governs untrusted and model-generated Reagent execution within `athanor-alembic`.

---

## 🎯 Core Technical Invariants

1. **Instruction Fuel Metering**:
   - Every execution Store must be initialized with an explicit fuel budget (`store.set_fuel(limit)`).
   - `wasm_config.consume_fuel(true)` must be enabled on the shared `wasmtime::Engine`.
   - Infinite loops or unbounded iterations MUST trap deterministically with `Trap::OutOfFuel` in $<15$ milliseconds without crashing or hanging the host process.

2. **Linear Memory Envelope**:
   - Guest linear memory is constrained by explicit allocation limits (default: 64 MB).
   - Any guest attempt to grow memory past the limit must trap cleanly.

3. **Zero-Access WASI**:
   - Default WASI context must not grant filesystem paths, network sockets, or process environment variables to untrusted tools.
   - Host-guest communication occurs strictly through linear memory JSON serialization.

---

## 🌉 Host-Guest Linear Memory Exchange Protocol

Communication between the host Rust runtime and the guest WASM module follows a clean 3-function ABI:

```text
Host                                                      Guest WASM Module
 │                                                              │
 ├── 1. Serialize input payload to JSON bytes                   │
 ├── 2. Call exported `allocate(size)` ────────────────────────►├── Allocates buffer in linear memory
 │   ◄──────────────────────────────────────────────────────────┴── Returns `guest_ptr`
 ├── 3. Write JSON bytes into guest memory at `guest_ptr`       │
 ├── 4. Call exported `transmute(guest_ptr, size)` ────────────►├── Parses JSON, executes business logic
 │                                                              │   Serializes output JSON to buffer
 │   ◄──────────────────────────────────────────────────────────┴── Returns packed u64 `(out_ptr << 32) | out_len`
 ├── 5. Unpack `out_ptr` and `out_len`                          │
 ├── 6. Read output JSON bytes from guest memory                │
 ├── 7. Call exported `deallocate(out_ptr, out_len)` ──────────►├── Frees guest output buffer
 └── 8. Parse output bytes into `serde_json::Value`             │
```

### ABI Function Signatures
```wat
;; Memory allocation helper
(func (export "allocate") (param $size i32) (result i32) ...)

;; Memory deallocation helper
(func (export "deallocate") (param $ptr i32) (param $size i32) ...)

;; Capability execution entrypoint
(func (export "transmute") (param $ptr i32) (param $len i32) (result i64) ...)
```

---

## 🛡️ Fuel Metering & Trap Handling Pattern

```rust
use wasmtime::*;
use athanor_core::reagent::ReagentError;

let mut config = Config::new();
config.consume_fuel(true);
let engine = Engine::new(&config)?;

let mut store = Store::new(&engine, ());
store.set_fuel(100_000_000)?; // 100M instructions

// Execute guest entrypoint
match transmute_fn.call_async(&mut store, (ptr, len)).await {
    Ok(packed) => {
        // Read remaining fuel to audit consumption
        let remaining_fuel = store.get_fuel().unwrap_or(0);
        let consumed = 100_000_000 - remaining_fuel;
        // Process packed (out_ptr, out_len)...
    }
    Err(trap) => {
        if let Some(Trap::OutOfFuel) = trap.downcast_ref::<Trap>() {
            return Err(ReagentError::Fatal("WASM fuel exhausted: execution aborted".into()));
        }
        return Err(ReagentError::Fatal(format!("WASM trap: {trap}")));
    }
}
```

---

## 📋 Technical References
- [WASM ABI Specification](./references/wasm_abi_spec.md) — Exact byte layout, pointer packing, and error codes.
- [WAT Test Fixtures](./references/wat_fixtures.md) — Ready-to-compile WebAssembly Text fixtures for tests (infinite loops, echo, math).
