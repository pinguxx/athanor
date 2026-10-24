//! Configuration presets and execution limits for the Alembic WebAssembly sandbox.

/// Execution parameters and containment boundaries for WebAssembly guest modules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlembicConfig {
    /// Maximum allowable linear memory in bytes (default: 64 MB = 67,108,864).
    pub max_memory_bytes: usize,
    /// Instruction fuel budget allocated per execution run (default: 100,000,000 instructions).
    pub fuel_limit: u64,
    /// Whether zero-access WASI Preview 1 sandboxing is enabled (default: true).
    pub wasi_enabled: bool,
}

impl Default for AlembicConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024,
            fuel_limit: 100_000_000,
            wasi_enabled: true,
        }
    }
}

impl AlembicConfig {
    /// Construct a new configuration initialized with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum linear memory envelope in bytes.
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Set the instruction fuel quota per execution.
    pub fn with_fuel_limit(mut self, fuel: u64) -> Self {
        self.fuel_limit = fuel;
        self
    }

    /// Toggle WASI Preview 1 containment.
    pub fn with_wasi(mut self, enabled: bool) -> Self {
        self.wasi_enabled = enabled;
        self
    }
}
