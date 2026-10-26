//! Athanor Atelier: Embedded Dev Studio visualizer and observability server.
//!
//! Provides an embedded HTTP and Server-Sent Events (SSE) server for inspecting
//! registered Artificers, Opuses, and Reagents, streaming execution traces in real time,
//! serving pre-compiled web dashboards from memory, and resuming suspended workflows.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod assets;
pub mod error;
pub mod registry;
pub mod server;
pub mod telemetry;

pub use assets::{static_asset_handler, StaticAssets};
pub use error::AtelierError;
pub use registry::{
    ArtificerSummary, AtelierRegistry, OpusSummary, ReagentSummary, RegistryResponse,
};
pub use server::{build_router, AtelierServer, BoundAtelierServer, ResumeRequestBody, ServerState};
pub use telemetry::{AtelierEvent, AtelierTracingLayer, TelemetryBroadcaster};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::error::AtelierError;
    pub use crate::registry::{
        ArtificerSummary, AtelierRegistry, OpusSummary, ReagentSummary, RegistryResponse,
    };
    pub use crate::server::{AtelierServer, BoundAtelierServer, ResumeRequestBody};
    pub use crate::telemetry::{AtelierEvent, AtelierTracingLayer, TelemetryBroadcaster};
}
