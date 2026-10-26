//! # Athanor Crucible
//!
//! SQLite-backed event sourcing, transactional checkpointing, and time-travel replay engine
//! for the Athanor agentic framework.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Crucible SQLite storage engine and query operations.
pub mod engine;
/// Crucible error hierarchy.
pub mod error;
/// Persistent data record representations.
pub mod record;

pub use engine::CrucibleEngine;
pub use error::CrucibleError;
pub use record::{OpusRunRecord, SuspendedTokenRecord, VesselEventRecord};
