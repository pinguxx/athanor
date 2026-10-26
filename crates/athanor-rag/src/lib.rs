#![recursion_limit = "512"]
#![deny(missing_docs)]

//! # Athanor Spagyric RAG
//!
//! Embedded, in-process Retrieval-Augmented Generation (RAG) subsystem for Athanor agents.
//!
//! Powered by:
//! - **FastEmbed**: In-process ONNX embeddings via `BGESmallENV15`.
//! - **LanceDB**: High-performance embedded columnar vector storage.
//! - **Recursive Splitters**: Delimiter-aware text chunking preserving paragraph and sentence boundaries.
//! - **Dynamic Search Reagent**: Native [`athanor_core::reagent::Reagent`] integration for Artificer reasoning loops.

pub mod document;
pub mod error;
pub mod index;
pub mod reagent;
pub mod splitter;

pub use document::{Document, SearchResult};
pub use error::RagError;
pub use index::SpagyricIndex;
pub use reagent::RagSearchReagent;
pub use splitter::RecursiveCharacterSplitter;

/// Convenience prelude re-exporting key RAG types and traits.
pub mod prelude {
    pub use crate::document::{Document, SearchResult};
    pub use crate::error::RagError;
    pub use crate::index::SpagyricIndex;
    pub use crate::reagent::RagSearchReagent;
    pub use crate::splitter::RecursiveCharacterSplitter;
}
