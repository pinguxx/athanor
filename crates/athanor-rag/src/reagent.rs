//! Dynamic search reagent exposing RAG capabilities to Artificer agents.

use async_trait::async_trait;
use athanor_core::reagent::{Reagent, ReagentError};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::index::SpagyricIndex;

const DEFAULT_REAGENT_NAME: &str = "rag_search";
const DEFAULT_REAGENT_DESCRIPTION: &str =
    "Searches the embedded vector knowledge base for relevant passages given a natural language query";

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    limit: Option<usize>,
}

/// Dynamic executable Reagent exposing semantic retrieval over a [`SpagyricIndex`].
pub struct RagSearchReagent {
    index: Arc<SpagyricIndex>,
    name: &'static str,
    description: &'static str,
}

impl RagSearchReagent {
    /// Construct a new `RagSearchReagent` wrapping a vector index with default name and description.
    pub fn new(index: Arc<SpagyricIndex>) -> Self {
        Self {
            index,
            name: DEFAULT_REAGENT_NAME,
            description: DEFAULT_REAGENT_DESCRIPTION,
        }
    }

    /// Construct a new `RagSearchReagent` with custom capability name and description.
    pub fn with_details(
        index: Arc<SpagyricIndex>,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            index,
            name,
            description,
        }
    }
}

#[async_trait]
impl Reagent for RagSearchReagent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language search query or keywords"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of relevant results to return (default: 5)"
                }
            },
            "required": ["query"]
        })
    }

    fn output_schema(&self) -> Value {
        serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "document": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "content": { "type": "string" },
                            "metadata": { "type": "object" }
                        },
                        "required": ["id", "content", "metadata"]
                    },
                    "score": {
                        "type": "number",
                        "description": "Cosine similarity score between 0.0 and 1.0"
                    }
                },
                "required": ["document", "score"]
            }
        })
    }

    async fn transmute(&self, input: Value) -> Result<Value, ReagentError> {
        let params: SearchParams = serde_json::from_value(input).map_err(|e| {
            ReagentError::invalid_input(format!("Invalid parameters for RAG search: {e}"))
        })?;

        let limit = params.limit.unwrap_or(5);
        let results = self
            .index
            .search(&params.query, limit)
            .await
            .map_err(ReagentError::from)?;

        let serialized = serde_json::to_value(results)?;
        Ok(serialized)
    }
}
