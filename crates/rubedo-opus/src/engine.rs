use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::OpusContext;
use crate::error::OpusError;
use crate::vessel::VesselStatus;
use crate::workflow::Opus;
use rubedo_crucible::CrucibleEngine;

/// Outcome result of executing or resuming an Opus workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpusRunResult {
    /// Workflow ran to completion and transmuted final state.
    Transmuted {
        /// Workflow run identifier.
        run_id: String,
        /// Final mutated state JSON snapshot.
        final_state: Value,
    },
    /// Workflow halted at a suspended Vessel step awaiting external resumption.
    Suspended {
        /// Workflow run identifier.
        run_id: String,
        /// Identifier of the Vessel node that suspended execution.
        vessel_id: String,
        /// Unforgeable token required to resume workflow.
        resume_token: String,
    },
    /// Workflow encountered a non-recoverable error.
    Corrupted {
        /// Workflow run identifier.
        run_id: String,
        /// Identifier of the Vessel node where corruption occurred.
        vessel_id: String,
        /// Diagnostic error description.
        error: String,
    },
}

impl OpusRunResult {
    /// Return the run ID associated with this result.
    pub fn run_id(&self) -> &str {
        match self {
            Self::Transmuted { run_id, .. } => run_id,
            Self::Suspended { run_id, .. } => run_id,
            Self::Corrupted { run_id, .. } => run_id,
        }
    }

    /// Returns `true` if the workflow completed successfully (`Transmuted`).
    pub fn is_transmuted(&self) -> bool {
        matches!(self, Self::Transmuted { .. })
    }

    /// Returns `true` if the workflow is currently `Suspended`.
    pub fn is_suspended(&self) -> bool {
        matches!(self, Self::Suspended { .. })
    }

    /// Returns `true` if the workflow failed (`Corrupted`).
    pub fn is_corrupted(&self) -> bool {
        matches!(self, Self::Corrupted { .. })
    }
}

/// The state machine orchestrator coordinating durable Vessel execution and SQLite checkpoints.
pub struct OpusEngine {
    crucible: Arc<CrucibleEngine>,
}

impl OpusEngine {
    /// Initialize an engine wired to a Crucible SQLite event store.
    pub fn new(crucible: Arc<CrucibleEngine>) -> Self {
        Self { crucible }
    }

    /// Execute an Opus workflow DAG from the beginning.
    pub async fn execute(
        &self,
        opus: &Opus,
        run_id_override: Option<&str>,
    ) -> Result<OpusRunResult, OpusError> {
        let run_id = run_id_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // 1. Commit initial run record into Crucible
        self.crucible
            .insert_run(&run_id, &opus.name, &opus.initial_state)
            .await?;

        // 2. Initialize execution context
        let mut ctx = OpusContext::new(&run_id, opus.initial_state.clone());

        // 3. Execute steps from beginning
        self.run_from_step(opus, &mut ctx, 0).await
    }

    /// Resume a suspended Opus workflow run from a validated token with external payload.
    pub async fn resume(
        &self,
        opus: &Opus,
        run_id: &str,
        token: &str,
        resume_payload: Value,
    ) -> Result<OpusRunResult, OpusError> {
        // 1. Atomically consume token from SQLite
        let (token_run_id, suspended_vessel_id) = self.crucible.consume_suspended_token(token).await?;

        if token_run_id != run_id {
            return Err(OpusError::Resume(format!(
                "Resume token belongs to run '{token_run_id}', but '{run_id}' was requested"
            )));
        }

        // 2. Hydrate latest state snapshot from Crucible
        let run_record = self.crucible.get_run(run_id).await?;

        // 3. Locate suspended Vessel index in workflow DAG
        let step_idx = opus
            .steps
            .iter()
            .position(|s| s.id() == suspended_vessel_id)
            .ok_or_else(|| {
                OpusError::Resume(format!(
                    "Suspended vessel '{suspended_vessel_id}' was not found in Opus '{}'",
                    opus.name
                ))
            })?;

        // 4. Record resumption event in journal
        self.crucible
            .record_vessel_event(run_id, &suspended_vessel_id, "Resumed", Some(&resume_payload))
            .await?;

        // 5. Hydrate context and inject resume payload
        let mut ctx = OpusContext::new(run_id, run_record.current_state);
        ctx.set_resume_payload(resume_payload);

        // 6. Resume execution from the step immediately following the suspended node
        self.run_from_step(opus, &mut ctx, step_idx + 1).await
    }

    async fn run_from_step(
        &self,
        opus: &Opus,
        ctx: &mut OpusContext,
        start_idx: usize,
    ) -> Result<OpusRunResult, OpusError> {
        let run_id = ctx.run_id().to_string();

        for idx in start_idx..opus.steps.len() {
            let step = &opus.steps[idx];
            let vessel_id = step.id();

            // Pre-execution checkpoint
            self.crucible
                .record_vessel_event(&run_id, vessel_id, "Active", None)
                .await?;
            self.crucible
                .update_run_state(&run_id, "Active", ctx.state())
                .await?;

            // Execute the Vessel node
            let status = match step.distill(ctx).await {
                Ok(s) => s,
                Err(err) => {
                    let err_msg = err.to_string();
                    self.crucible
                        .record_vessel_event(
                            &run_id,
                            vessel_id,
                            "Corrupted",
                            Some(&serde_json::json!({ "error": &err_msg })),
                        )
                        .await?;
                    self.crucible
                        .update_run_state(&run_id, "Failed", ctx.state())
                        .await?;

                    return Ok(OpusRunResult::Corrupted {
                        run_id,
                        vessel_id: vessel_id.to_string(),
                        error: err_msg,
                    });
                }
            };

            // Post-execution checkpoint handling
            match status {
                VesselStatus::Transmuted { output } => {
                    self.crucible
                        .record_vessel_event(&run_id, vessel_id, "Transmuted", Some(&output))
                        .await?;
                    self.crucible
                        .update_run_state(&run_id, "Active", ctx.state())
                        .await?;
                }
                VesselStatus::Suspended { resume_token } => {
                    self.crucible
                        .register_suspended_token(&resume_token, &run_id, vessel_id)
                        .await?;
                    self.crucible
                        .record_vessel_event(
                            &run_id,
                            vessel_id,
                            "Suspended",
                            Some(&serde_json::json!({ "resume_token": &resume_token })),
                        )
                        .await?;
                    self.crucible
                        .update_run_state(&run_id, "Suspended", ctx.state())
                        .await?;

                    return Ok(OpusRunResult::Suspended {
                        run_id,
                        vessel_id: vessel_id.to_string(),
                        resume_token,
                    });
                }
                VesselStatus::Corrupted { error } => {
                    self.crucible
                        .record_vessel_event(
                            &run_id,
                            vessel_id,
                            "Corrupted",
                            Some(&serde_json::json!({ "error": &error })),
                        )
                        .await?;
                    self.crucible
                        .update_run_state(&run_id, "Failed", ctx.state())
                        .await?;

                    return Ok(OpusRunResult::Corrupted {
                        run_id,
                        vessel_id: vessel_id.to_string(),
                        error,
                    });
                }
                VesselStatus::Pending | VesselStatus::Active => {
                    // Step remained in progress without state transition
                }
            }
        }

        // Complete workflow
        let final_state = ctx.state().clone();
        self.crucible
            .update_run_state(&run_id, "Completed", &final_state)
            .await?;

        Ok(OpusRunResult::Transmuted {
            run_id,
            final_state,
        })
    }
}
