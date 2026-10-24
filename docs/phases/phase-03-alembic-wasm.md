# Phase 3: Alembic WASM Sandbox Runtime

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-alembic`
- **Primary Goal**: Provide a secure, in-process WebAssembly sandbox using `wasmtime` and `wasmtime-wasi`. Model-generated or untrusted dynamic tools execute with strict linear memory envelopes (e.g. 64 MB) and instruction fuel limits (e.g. 100M instructions), aborting infinite loops within $<15$ milliseconds without crashing the host.

---

## 2. Detailed Task Breakdown

### Task 3.1: Scaffold `rubedo-alembic`
- Create `crates/rubedo-alembic/Cargo.toml` with dependencies:
  - `wasmtime = "25.0"`
  - `wasmtime-wasi = "25.0"`
  - `rubedo-core`
  - `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `anyhow`

### Task 3.2: Implement `AlembicConfig` and `AlembicError`
- In `crates/rubedo-alembic/src/config.rs`:
  ```rust
  #[derive(Clone, Debug)]
  pub struct AlembicConfig {
      pub max_memory_bytes: usize, // e.g., 64 * 1024 * 1024 (64MB)
      pub fuel_limit: u64,         // e.g., 100_000_000 instructions
  }

  impl Default for AlembicConfig {
      fn default() -> Self {
          Self {
              max_memory_bytes: 64 * 1024 * 1024,
              fuel_limit: 100_000_000,
          }
      }
  }
  ```
- In `crates/rubedo-alembic/src/error.rs`:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum AlembicError {
      #[error("Fuel exhausted: execution exceeded instruction limit")]
      FuelExhausted,
      #[error("Memory limit exceeded: requested {requested} bytes, max {max} bytes")]
      MemoryLimitExceeded { requested: usize, max: usize },
      #[error("Guest instantiation error: {0}")]
      Instantiation(String),
      #[error("WASM execution trapped: {0}")]
      Trap(String),
      #[error("Serialization error: {0}")]
      Serialization(String),
  }
  ```

### Task 3.3: Host-Guest Memory Exchange Protocol
- In `crates/rubedo-alembic/src/memory.rs`:
  - Define guest ABI convention:
    - `allocate(size: u32) -> u32` (allocates buffer in guest memory)
    - `deallocate(ptr: u32, size: u32)` (frees buffer)
    - `transmute(ptr: u32, len: u32) -> u64` (returns packed `(ptr << 32) | len`)
  - Implement host helpers to write serialized JSON into guest memory and read output buffer back into `serde_json::Value`.

### Task 3.4: Implement `AlembicSandbox`
- In `crates/rubedo-alembic/src/sandbox.rs`:
  ```rust
  use wasmtime::*;
  use crate::config::AlembicConfig;
  use crate::error::AlembicError;
  use serde_json::Value;

  pub struct AlembicSandbox {
      engine: Engine,
      config: AlembicConfig,
  }

  impl AlembicSandbox {
      pub fn new(config: AlembicConfig) -> Result<Self, AlembicError> {
          let mut wasm_config = Config::new();
          wasm_config.consume_fuel(true);
          let engine = Engine::new(&wasm_config)
              .map_err(|e| AlembicError::Instantiation(e.to_string()))?;
          Ok(Self { engine, config })
      }

      pub async fn execute_raw(
          &self,
          wasm_binary: &[u8],
          input: Value,
      ) -> Result<Value, AlembicError> {
          // Initialize store with fuel limit
          let mut store = Store::new(&self.engine, ());
          store.set_fuel(self.config.fuel_limit)
              .map_err(|e| AlembicError::Trap(e.to_string()))?;

          // Compile and instantiate module
          let module = Module::new(&self.engine, wasm_binary)
              .map_err(|e| AlembicError::Instantiation(e.to_string()))?;
          let instance = Instance::new(&mut store, &module, &[])
              .map_err(|e| AlembicError::Instantiation(e.to_string()))?;

          // Execute with fuel tracking
          // Map Trap::OutOfFuel to AlembicError::FuelExhausted
          todo!()
      }
  }
  ```

### Task 3.5: Wrap WASM Modules as `rubedo_core::Reagent`
- In `crates/rubedo-alembic/src/reagent.rs`:
  - Implement `WasmReagent` which wraps a compiled WASM binary and implements `rubedo_core::reagent::Reagent`.
  - Maps `AlembicError::FuelExhausted` into `ReagentError::Fatal("Fuel exhausted".into())`.

---

## 3. Verification & Acceptance Criteria

### Unit & Integration Test
Create `crates/rubedo-alembic/tests/fuel_exhaustion_test.rs`:
1. Compile a minimal WAT (WebAssembly Text) guest containing an intentional infinite loop:
   ```wat
   (module
     (func (export "run")
       (loop (br 0))
     )
   )
   ```
2. Execute the guest using `AlembicSandbox` with `fuel_limit = 100_000`.
3. Assert:
   - Host execution returns `Err(AlembicError::FuelExhausted)`.
   - The test completes in $<15$ milliseconds.
   - Host process does not crash or hang.

### Execution Command
```bash
cargo test -p rubedo-alembic
```
