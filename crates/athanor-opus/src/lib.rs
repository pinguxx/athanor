//! # Athanor Opus
//!
//! Durable directed acyclic graph (DAG) workflow engine with event-sourced checkpointing
//! for the Athanor agentic framework.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Shared workflow state memory context.
pub mod context;
/// Workflow execution engine and state machine orchestrator.
pub mod engine;
/// Opus error hierarchy.
pub mod error;
/// Vessel trait and step status models.
pub mod vessel;
/// Opus workflow definition and builder.
pub mod workflow;

/// Prelude module for convenient imports of standard Opus symbols.
pub mod prelude {
    pub use crate::context::OpusContext;
    pub use crate::engine::{OpusEngine, OpusRunResult};
    pub use crate::error::OpusError;
    pub use crate::vessel::{Vessel, VesselError, VesselStatus};
    pub use crate::workflow::{Opus, OpusBuilder};
}

pub use context::OpusContext;
pub use engine::{OpusEngine, OpusRunResult};
pub use error::OpusError;
pub use vessel::{Vessel, VesselError, VesselStatus};
pub use workflow::{Opus, OpusBuilder};
