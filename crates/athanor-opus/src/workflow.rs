use serde_json::Value;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use crate::error::OpusError;
use crate::vessel::Vessel;

/// A directed acyclic graph (DAG) workflow of Vessel execution steps.
#[derive(Clone)]
pub struct Opus {
    /// Name or title of the workflow.
    pub name: String,
    /// Sequential or DAG execution steps.
    pub steps: Vec<Arc<dyn Vessel>>,
    /// Default initial state variables.
    pub initial_state: Value,
}

impl fmt::Debug for Opus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let step_ids: Vec<&str> = self.steps.iter().map(|s| s.id()).collect();
        f.debug_struct("Opus")
            .field("name", &self.name)
            .field("step_ids", &step_ids)
            .field("initial_state", &self.initial_state)
            .finish()
    }
}

impl Opus {
    /// Create a fluent builder for configuring an `Opus` workflow.
    pub fn builder(name: impl Into<String>) -> OpusBuilder {
        OpusBuilder::new(name)
    }
}

/// Fluent builder for constructing an [`Opus`] workflow.
pub struct OpusBuilder {
    name: String,
    steps: Vec<Arc<dyn Vessel>>,
    initial_state: Option<Value>,
}

impl fmt::Debug for OpusBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let step_ids: Vec<&str> = self.steps.iter().map(|s| s.id()).collect();
        f.debug_struct("OpusBuilder")
            .field("name", &self.name)
            .field("step_ids", &step_ids)
            .field("initial_state", &self.initial_state)
            .finish()
    }
}

impl OpusBuilder {
    /// Initialize a builder with the workflow name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            initial_state: None,
        }
    }

    /// Add a step to the workflow DAG.
    pub fn step(mut self, step: Arc<dyn Vessel>) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to the workflow.
    pub fn steps(mut self, steps: impl IntoIterator<Item = Arc<dyn Vessel>>) -> Self {
        self.steps.extend(steps);
        self
    }

    /// Set initial state JSON snapshot.
    pub fn initial_state(mut self, state: Value) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Finalize and build the [`Opus`] workflow.
    pub fn build(self) -> Result<Opus, OpusError> {
        if self.steps.is_empty() {
            return Err(OpusError::Topology(
                "An Opus workflow must contain at least one Vessel step".into(),
            ));
        }

        // Validate unique Vessel identifiers
        let mut seen = HashSet::new();
        for step in &self.steps {
            let id = step.id();
            if !seen.insert(id) {
                return Err(OpusError::Topology(format!(
                    "Duplicate Vessel step identifier detected: '{id}'"
                )));
            }
        }

        let initial_state = self.initial_state.unwrap_or_else(|| serde_json::json!({}));

        Ok(Opus {
            name: self.name,
            steps: self.steps,
            initial_state,
        })
    }
}
