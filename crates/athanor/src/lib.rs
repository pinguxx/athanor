//! # Athanor: Ironclad Agentic Framework for Rust
//!
//! Athanor is an enterprise-grade agentic framework engineered in Rust. Named after the
//! cosmic, self-feeding furnace of the alchemists, Athanor distills chaotic,
//! non-deterministic LLM streams into deterministic, verified, and durable state
//! transitions using Rust’s compile-time safety and async runtime.
//!
//! ## Subsystem Modules
//!
//! - `core`: Foundational traits (`Reagent`, `Artificer`, `AthanorContext`, `Envelope`).
//! - `crucible`: SQLite-backed event sourcing and persistent state checkpoints (feature: `crucible`).
//! - `opus`: Resilient DAG workflow engine with tokenized pause/resume (feature: `opus`).
//! - `alembic`: Fuel-metered WebAssembly sandbox runtime (feature: `alembic`).
//! - `mcp`: Bidirectional Model Context Protocol (MCP 2024-11-05+) gateway (feature: `mcp`).
//! - `rag`: Spagyric embedded in-process vector store with LanceDB and FastEmbed (feature: `rag`).
//! - `atelier`: Embedded Dev Studio visualizer and SSE telemetry server (feature: `atelier`).
//! - `evals`: Verification harness, token guardrails, and LLM judges (feature: `evals`).
//!
//! ## Quickstart
//!
//! ```rust
//! use athanor::prelude::*;
//!
//! #[reagent(name = "calculator")]
//! async fn calculate(a: f64, b: f64) -> Result<f64, ReagentError> {
//!     Ok(a + b)
//! }
//! ```

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Core traits, agent types, and execution context.
pub mod core {
    pub use athanor_core::*;
}

// Re-export core items directly at root
pub use athanor_core::artificer::{Artificer, ArtificerBuilder};
pub use athanor_core::async_trait;
pub use athanor_core::context::{
    AthanorContext, Message, MessageRole, TokenUsage, ToolExecutionRecord,
};
pub use athanor_core::envelope::Envelope;
pub use athanor_core::error::AthanorError;
pub use athanor_core::reagent::{Reagent, ReagentError};

/// Procedural macros for synthesizing Reagents and test benchmarks.
#[cfg(feature = "macros")]
pub mod macros {
    pub use athanor_macros::*;
}

#[cfg(feature = "macros")]
pub use athanor_macros::reagent;

#[cfg(all(feature = "macros", feature = "evals"))]
pub use athanor_macros::artificer_test;

/// SQLite event sourcing, transactional checkpoints, and time-travel replay.
#[cfg(feature = "crucible")]
pub mod crucible {
    pub use athanor_crucible::*;
}

/// Durable directed acyclic graph (DAG) workflow engine.
#[cfg(feature = "opus")]
pub mod opus {
    pub use athanor_opus::*;
}

/// In-process WebAssembly sandbox runtime with instruction fuel metering.
#[cfg(feature = "alembic")]
pub mod alembic {
    pub use athanor_alembic::*;
}

/// Bidirectional Model Context Protocol (MCP 2024-11-05+) bridge.
#[cfg(feature = "mcp")]
pub mod mcp {
    pub use athanor_mcp::*;
}

/// Embedded Spagyric in-process vector store and semantic search.
#[cfg(feature = "rag")]
pub mod rag {
    pub use athanor_rag::*;
}

/// Embedded Dev Studio visualizer and SSE telemetry server.
#[cfg(feature = "atelier")]
pub mod atelier {
    pub use athanor_atelier::*;
}

/// Verification harness, deterministic metrics, and evaluation guardrails.
#[cfg(feature = "evals")]
pub mod evals {
    pub use athanor_evals::*;
}

/// Prelude module for convenient imports across agent pipelines.
pub mod prelude {
    pub use athanor_core::artificer::{Artificer, ArtificerBuilder};
    pub use athanor_core::async_trait;
    pub use athanor_core::context::{
        AthanorContext, Message, MessageRole, TokenUsage, ToolExecutionRecord,
    };
    pub use athanor_core::envelope::Envelope;
    pub use athanor_core::error::AthanorError;
    pub use athanor_core::reagent::{Reagent, ReagentError};

    #[cfg(feature = "macros")]
    pub use athanor_macros::reagent;

    #[cfg(all(feature = "macros", feature = "evals"))]
    pub use athanor_macros::artificer_test;

    #[cfg(feature = "crucible")]
    pub use athanor_crucible::CrucibleEngine;

    #[cfg(feature = "opus")]
    pub use athanor_opus::prelude::*;

    #[cfg(feature = "alembic")]
    pub use athanor_alembic::prelude::*;

    #[cfg(feature = "mcp")]
    pub use athanor_mcp::prelude::*;

    #[cfg(feature = "rag")]
    pub use athanor_rag::prelude::*;

    #[cfg(feature = "atelier")]
    pub use athanor_atelier::prelude::*;

    #[cfg(feature = "evals")]
    pub use athanor_evals::prelude::*;
}
