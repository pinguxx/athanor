//! In-memory component registry for registered Artificers, Opuses, and Reagents.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use athanor_core::{Artificer, Reagent};
use athanor_opus::Opus;

/// Summary representation of an Artificer for dev tools and UI inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtificerSummary {
    /// Identifier or persona name.
    pub name: String,
    /// System prompt instructions.
    pub instructions: String,
    /// Targeted LLM model identifier.
    pub model: String,
    /// Maximum allowed iterative reasoning steps.
    pub max_steps: usize,
    /// List of executable Reagent capability names attached to this Artificer.
    pub reagents: Vec<String>,
}

/// Summary representation of an Opus workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpusSummary {
    /// Workflow name.
    pub name: String,
    /// Ordered identifiers of the Vessel steps composing the workflow.
    pub step_ids: Vec<String>,
    /// Default initial state variables.
    pub initial_state: Value,
}

/// Summary representation of an executable Reagent tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReagentSummary {
    /// Canonical unique tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Expected JSON Schema for input arguments.
    pub input_schema: Value,
    /// Returned JSON Schema for output payload.
    pub output_schema: Value,
}

/// Full registry payload returned by the `/api/registry` HTTP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryResponse {
    /// Registered autonomous Artificers.
    pub artificers: Vec<ArtificerSummary>,
    /// Registered durable Opus workflows.
    pub opuses: Vec<OpusSummary>,
    /// Registered executable Reagent capabilities.
    pub reagents: Vec<ReagentSummary>,
}

/// Central in-memory registry holding registered agents, workflows, and tools.
#[derive(Default, Clone)]
pub struct AtelierRegistry {
    artificers: Arc<RwLock<HashMap<String, ArtificerSummary>>>,
    opuses: Arc<RwLock<HashMap<String, Arc<Opus>>>>,
    opus_summaries: Arc<RwLock<HashMap<String, OpusSummary>>>,
    reagents: Arc<RwLock<HashMap<String, ReagentSummary>>>,
}

impl AtelierRegistry {
    /// Initialize an empty component registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an active `Artificer` instance by extracting its metadata.
    pub fn register_artificer(&self, artificer: &Artificer) {
        let summary = ArtificerSummary {
            name: artificer.name.clone(),
            instructions: artificer.instructions.clone(),
            model: artificer.model.clone(),
            max_steps: artificer.max_steps,
            reagents: artificer
                .reagent_names()
                .into_iter()
                .map(String::from)
                .collect(),
        };
        self.register_artificer_summary(summary);
    }

    /// Register an `ArtificerSummary` directly.
    pub fn register_artificer_summary(&self, summary: ArtificerSummary) {
        let mut map = write_lock(&self.artificers);
        map.insert(summary.name.clone(), summary);
    }

    /// Register a durable `Opus` workflow definition.
    pub fn register_opus(&self, opus: Arc<Opus>) {
        let summary = OpusSummary {
            name: opus.name.clone(),
            step_ids: opus.steps.iter().map(|s| s.id().to_string()).collect(),
            initial_state: opus.initial_state.clone(),
        };
        {
            let mut op_map = write_lock(&self.opuses);
            op_map.insert(opus.name.clone(), opus);
        }
        {
            let mut sum_map = write_lock(&self.opus_summaries);
            sum_map.insert(summary.name.clone(), summary);
        }
    }

    /// Register an executable `Reagent` tool by extracting its metadata.
    pub fn register_reagent(&self, reagent: &dyn Reagent) {
        let summary = ReagentSummary {
            name: reagent.name().to_string(),
            description: reagent.description().to_string(),
            input_schema: reagent.input_schema(),
            output_schema: reagent.output_schema(),
        };
        self.register_reagent_summary(summary);
    }

    /// Register a `ReagentSummary` directly.
    pub fn register_reagent_summary(&self, summary: ReagentSummary) {
        let mut map = write_lock(&self.reagents);
        map.insert(summary.name.clone(), summary);
    }

    /// Retrieve an `Opus` workflow definition by name.
    pub fn get_opus(&self, name: &str) -> Option<Arc<Opus>> {
        let map = read_lock(&self.opuses);
        map.get(name).cloned()
    }

    /// Generate a point-in-time snapshot of all registered components.
    pub fn snapshot(&self) -> RegistryResponse {
        let artificers: Vec<ArtificerSummary> = {
            let map = read_lock(&self.artificers);
            let mut list: Vec<ArtificerSummary> = map.values().cloned().collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            list
        };

        let opuses: Vec<OpusSummary> = {
            let map = read_lock(&self.opus_summaries);
            let mut list: Vec<OpusSummary> = map.values().cloned().collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            list
        };

        let reagents: Vec<ReagentSummary> = {
            let map = read_lock(&self.reagents);
            let mut list: Vec<ReagentSummary> = map.values().cloned().collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            list
        };

        RegistryResponse {
            artificers,
            opuses,
            reagents,
        }
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
