//! Sandboxed execution engine governing WebAssembly fuel metering and memory isolation.

use serde_json::Value;
use std::sync::Arc;
use wasmtime::{Config, Engine, Instance, Linker, Module, ResourceLimiter, Store};
use wasmtime_wasi::preview1::{add_to_linker_sync, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

use crate::config::AlembicConfig;
use crate::error::AlembicError;
use crate::memory::{execute_exchange, map_trap};

/// Internal Store execution state holding WASI context and memory limits.
pub struct StoreState {
    /// WASI Preview 1 isolation context.
    pub wasi: WasiP1Ctx,
    /// Maximum linear memory boundary in bytes.
    pub max_memory_bytes: usize,
}

impl StoreState {
    /// Construct store state with configured memory limit and zero-access WASI context.
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build_p1(),
            max_memory_bytes,
        }
    }
}

impl ResourceLimiter for StoreState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        if desired > self.max_memory_bytes {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> Result<bool, wasmtime::Error> {
        Ok(true)
    }
}

/// WebAssembly sandbox engine providing deterministic fuel metering and memory isolation.
#[derive(Clone)]
pub struct AlembicSandbox {
    engine: Engine,
    config: Arc<AlembicConfig>,
}

impl AlembicSandbox {
    /// Initialize a new sandbox engine configured with fuel metering and isolation boundaries.
    pub fn new(config: AlembicConfig) -> Result<Self, AlembicError> {
        let mut wasm_config = Config::new();
        wasm_config.consume_fuel(true);

        let engine =
            Engine::new(&wasm_config).map_err(|e| AlembicError::Instantiation(e.to_string()))?;

        Ok(Self {
            engine,
            config: Arc::new(config),
        })
    }

    /// Access the active sandbox configuration.
    pub fn config(&self) -> &AlembicConfig {
        &self.config
    }

    /// Access the underlying Wasmtime execution engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Execute a guest module implementing the linear memory exchange protocol.
    pub fn execute(&self, wasm_binary: &[u8], input: Value) -> Result<Value, AlembicError> {
        let input_bytes = serde_json::to_vec(&input)?;

        if input_bytes.len() > self.config.max_memory_bytes {
            return Err(AlembicError::MemoryLimitExceeded {
                requested: input_bytes.len(),
                max: self.config.max_memory_bytes,
            });
        }

        let (mut store, instance) = self.instantiate(wasm_binary)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| AlembicError::MissingExport("memory".into()))?;

        let output_bytes = execute_exchange(&instance, &mut store, &memory, &input_bytes)?;
        let output: Value = serde_json::from_slice(&output_bytes)?;

        Ok(output)
    }

    /// Execute a zero-argument exported function (e.g. "run") on a guest module.
    pub fn execute_fn(&self, wasm_binary: &[u8], export_name: &str) -> Result<(), AlembicError> {
        let (mut store, instance) = self.instantiate(wasm_binary)?;

        let func = instance
            .get_typed_func::<(), ()>(&mut store, export_name)
            .map_err(|_| AlembicError::MissingExport(export_name.into()))?;

        func.call(&mut store, ()).map_err(map_trap)?;

        Ok(())
    }

    /// Execute guest module automatically detecting whether to invoke "transmute" or "run".
    pub fn execute_raw(&self, wasm_binary: &[u8], input: Value) -> Result<Value, AlembicError> {
        let (mut store, instance) = self.instantiate(wasm_binary)?;

        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            if instance
                .get_typed_func::<(u32, u32), u64>(&mut store, "transmute")
                .is_ok()
            {
                let input_bytes = serde_json::to_vec(&input)?;
                if input_bytes.len() > self.config.max_memory_bytes {
                    return Err(AlembicError::MemoryLimitExceeded {
                        requested: input_bytes.len(),
                        max: self.config.max_memory_bytes,
                    });
                }
                let output_bytes = execute_exchange(&instance, &mut store, &memory, &input_bytes)?;
                let output: Value = serde_json::from_slice(&output_bytes)?;
                return Ok(output);
            }
        }

        if let Ok(run) = instance.get_typed_func::<(), ()>(&mut store, "run") {
            run.call(&mut store, ()).map_err(map_trap)?;
            return Ok(Value::Null);
        }

        Err(AlembicError::MissingExport("transmute or run".into()))
    }

    fn instantiate(
        &self,
        wasm_binary: &[u8],
    ) -> Result<(Store<StoreState>, Instance), AlembicError> {
        let state = StoreState::new(self.config.max_memory_bytes);
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| s);
        store
            .set_fuel(self.config.fuel_limit)
            .map_err(|e| AlembicError::Trap(e.to_string()))?;

        let mut linker = Linker::new(&self.engine);
        if self.config.wasi_enabled {
            add_to_linker_sync(&mut linker, |state: &mut StoreState| &mut state.wasi)
                .map_err(|e| AlembicError::Instantiation(e.to_string()))?;
        }

        let module = Module::new(&self.engine, wasm_binary)
            .map_err(|e| AlembicError::Instantiation(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| AlembicError::Instantiation(e.to_string()))?;

        Ok((store, instance))
    }
}
