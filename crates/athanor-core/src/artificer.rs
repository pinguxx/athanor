use serde_json::Value;
use std::fmt;
use std::sync::Arc;

use crate::error::AthanorError;
use crate::reagent::Reagent;

/// An autonomous reasoning loop executing over history, context, and reagents.
pub struct Artificer {
    /// Identifier or persona name of this Artificer.
    pub name: String,
    /// System instructions or prompting guidelines.
    pub instructions: String,
    /// Identifier of the target LLM model (e.g., "claude-3-5-sonnet", "gpt-4o").
    pub model: String,
    /// Available executable tools (Reagents) accessible to this Artificer.
    pub reagents: Vec<Arc<dyn Reagent>>,
    /// Maximum number of iterative reasoning steps before terminating.
    pub max_steps: usize,
}

impl fmt::Debug for Artificer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reagent_names: Vec<&str> = self.reagents.iter().map(|r| r.name()).collect();
        f.debug_struct("Artificer")
            .field("name", &self.name)
            .field("instructions", &self.instructions)
            .field("model", &self.model)
            .field("reagents", &reagent_names)
            .field("max_steps", &self.max_steps)
            .finish()
    }
}

impl Artificer {
    /// Create a fluent builder for configuring an `Artificer`.
    pub fn builder() -> ArtificerBuilder {
        ArtificerBuilder::default()
    }

    /// Add an executable Reagent capability to this Artificer.
    pub fn add_reagent(&mut self, reagent: Arc<dyn Reagent>) {
        self.reagents.push(reagent);
    }

    /// Check if a Reagent with the specified name is registered.
    pub fn has_reagent(&self, name: &str) -> bool {
        self.reagents.iter().any(|r| r.name() == name)
    }

    /// Retrieve a reference to a registered Reagent by name.
    pub fn get_reagent(&self, name: &str) -> Option<Arc<dyn Reagent>> {
        self.reagents.iter().find(|r| r.name() == name).cloned()
    }

    /// Return the list of registered Reagent names.
    pub fn reagent_names(&self) -> Vec<&'static str> {
        self.reagents.iter().map(|r| r.name()).collect()
    }

    /// Collect input JSON schemas for all registered Reagents.
    pub fn reagent_schemas(&self) -> Vec<Value> {
        self.reagents
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name(),
                    "description": r.description(),
                    "input_schema": r.input_schema(),
                    "output_schema": r.output_schema(),
                })
            })
            .collect()
    }
}

/// Fluent builder for constructing an [`Artificer`].
#[derive(Default)]
pub struct ArtificerBuilder {
    name: Option<String>,
    instructions: Option<String>,
    model: Option<String>,
    reagents: Vec<Arc<dyn Reagent>>,
    max_steps: Option<usize>,
}

impl ArtificerBuilder {
    /// Set the Artificer name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the system instructions for this Artificer.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set the model identifier.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Attach a single Reagent capability.
    pub fn reagent(mut self, reagent: Arc<dyn Reagent>) -> Self {
        self.reagents.push(reagent);
        self
    }

    /// Attach multiple Reagents.
    pub fn reagents(mut self, reagents: impl IntoIterator<Item = Arc<dyn Reagent>>) -> Self {
        self.reagents.extend(reagents);
        self
    }

    /// Set the maximum reasoning step budget.
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = Some(max_steps);
        self
    }

    /// Finalize and build the [`Artificer`].
    pub fn build(self) -> Result<Artificer, AthanorError> {
        let name = self
            .name
            .ok_or_else(|| AthanorError::Artificer("Artificer 'name' is required".into()))?;
        let instructions = self.instructions.unwrap_or_default();
        let model = self
            .model
            .ok_or_else(|| AthanorError::Artificer("Artificer 'model' is required".into()))?;
        let max_steps = self.max_steps.unwrap_or(25);

        Ok(Artificer {
            name,
            instructions,
            model,
            reagents: self.reagents,
            max_steps,
        })
    }
}
