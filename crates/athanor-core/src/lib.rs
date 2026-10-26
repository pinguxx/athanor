//! # Athanor Core
//!
//! Foundational traits, error hierarchies, agent containers, and context digestion
//! primitives for the Athanor agentic framework.
//!
//! Athanor guarantees `'static + Send + Sync` value semantics across all core interfaces,
//! freeing autonomous agent synthesizers from lifetime annotations.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Artificer agent representation and configuration builder.
pub mod artificer;
/// Execution context, conversation history, and token metrics.
pub mod context;
/// Zero-lifetime owned envelope containers.
pub mod envelope;
/// Core error hierarchy.
pub mod error;
/// Reagent traits, error models, and execution contracts.
pub mod reagent;

/// Prelude module for convenient imports of standard Athanor core symbols.
pub mod prelude {
    pub use crate::artificer::{Artificer, ArtificerBuilder};
    pub use crate::context::{
        AthanorContext, Message, MessageRole, TokenUsage, ToolExecutionRecord,
    };
    pub use crate::envelope::Envelope;
    pub use crate::error::AthanorError;
    pub use crate::reagent::{Reagent, ReagentError};
    pub use async_trait::async_trait;
}

pub use artificer::{Artificer, ArtificerBuilder};
pub use async_trait::async_trait;
pub use context::AthanorContext;
pub use envelope::Envelope;
pub use error::AthanorError;
pub use reagent::{Reagent, ReagentError};
