//! In-process WebAssembly sandbox runtime with instruction fuel metering and WASI for Athanor.
//!
//! Provides containment envelopes (default: 64 MB linear memory) and instruction fuel quotas
//! (default: 100,000,000 instructions) terminating unbounded iterations in $<15\text{ ms}$.

#![deny(missing_docs)]

pub mod config;
pub mod error;
pub mod memory;
pub mod reagent;
pub mod sandbox;

pub use config::AlembicConfig;
pub use error::AlembicError;
pub use reagent::WasmReagent;
pub use sandbox::AlembicSandbox;

/// Prelude re-exporting key types and traits for `athanor-alembic`.
pub mod prelude {
    pub use crate::config::AlembicConfig;
    pub use crate::error::AlembicError;
    pub use crate::reagent::WasmReagent;
    pub use crate::sandbox::AlembicSandbox;
}
