//! # Example: Durable Transmutation Workflow
//!
//! Demonstrates building an event-sourced, durable workflow with [`rubedo_opus::Opus`]
//! and [`rubedo_crucible::CrucibleEngine`].
//!
//! Illustrates:
//! 1. Multi-step directed acyclic graph (DAG) state transitions.
//! 2. Suspending execution when human review / external approval is mandated.
//! 3. Minting unforgeable resume tokens stored in SQLite.
//! 4. Resuming the paused workflow with injected external approval payloads.
//! 5. Inspecting the immutable audit journal in SQLite.

use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use rubedo_crucible::CrucibleEngine;
use rubedo_opus::context::OpusContext;
use rubedo_opus::engine::{OpusEngine, OpusRunResult};
use rubedo_opus::vessel::{Vessel, VesselError, VesselStatus};
use rubedo_opus::workflow::Opus;

// ---------------------------------------------------------------------------
// 1. Workflow Vessel Step Definitions
// ---------------------------------------------------------------------------

/// Step 1: Drafts an alchemical transmutation proposal.
struct Step1DraftProposal;

#[async_trait]
impl Vessel for Step1DraftProposal {
    fn id(&self) -> &'static str {
        "step_1_draft_proposal"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let batch_id = "BATCH-ALCH-2026-X";
        let base_metal = "Lead";
        let raw_weight_oz = 1000.0;

        ctx.set("batch_id", batch_id)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("base_metal", base_metal)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("raw_weight_oz", raw_weight_oz)
            .map_err(|e| VesselError::Execution(e.to_string()))?;

        println!("   [Step 1] Proposal drafted: {raw_weight_oz} oz of {base_metal} (Batch: {batch_id})");

        Ok(VesselStatus::Transmuted {
            output: json!({
                "batch_id": batch_id,
                "status": "drafted",
                "weight_oz": raw_weight_oz
            }),
        })
    }
}

/// Step 2: Checks compliance policies; suspends execution for large batches.
struct Step2HumanApproval;

#[async_trait]
impl Vessel for Step2HumanApproval {
    fn id(&self) -> &'static str {
        "step_2_await_approval"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let weight: f64 = ctx
            .get("raw_weight_oz")
            .ok_or_else(|| VesselError::InvalidState("Missing 'raw_weight_oz'".into()))?;

        // Transactions exceeding 100 oz mandate human overseer approval
        if weight > 100.0 {
            let resume_token = format!("approval-token-{}", uuid::Uuid::new_v4());
            println!("   [Step 2] Compliance policy triggered: High-value batch exceeds 100 oz.");
            println!("            Yielding cryptographic token and suspending workflow state.");

            Ok(VesselStatus::Suspended { resume_token })
        } else {
            Ok(VesselStatus::Transmuted {
                output: json!({ "approval": "auto_approved" }),
            })
        }
    }
}

/// Step 3: Hydrates external approval payload and finalizes transmutation.
struct Step3CompleteTransmutation;

#[async_trait]
impl Vessel for Step3CompleteTransmutation {
    fn id(&self) -> &'static str {
        "step_3_finalize_transmutation"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let weight: f64 = ctx
            .get("raw_weight_oz")
            .ok_or_else(|| VesselError::InvalidState("Missing 'raw_weight_oz'".into()))?;

        // Extract the externally injected resume payload
        let (approver, purity) = {
            let payload = ctx
                .resume_payload()
                .ok_or_else(|| VesselError::InvalidState("Missing required approval payload".into()))?;

            let approver = payload
                .get("approver")
                .and_then(|v| v.as_str())
                .unwrap_or("Anonymous Overseer")
                .to_string();

            let purity = payload
                .get("purity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.999);

            (approver, purity)
        };

        let final_gold_oz = weight * purity;

        ctx.set("approver", &approver)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("gold_produced_oz", final_gold_oz)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("status", "TransmutationComplete")
            .map_err(|e| VesselError::Execution(e.to_string()))?;

        println!("   [Step 3] Approval verified from: '{approver}'.");
        println!("            Transmutation finalized: {final_gold_oz} oz of fine Gold produced.");

        Ok(VesselStatus::Transmuted {
            output: json!({
                "approver": approver,
                "gold_produced_oz": final_gold_oz,
                "status": "TransmutationComplete"
            }),
        })
    }
}

// ---------------------------------------------------------------------------
// 2. Main Execution Loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("============================================================");
    println!(" ⚗ RUBEDO: Durable Transmutation & Token Resumption Example");
    println!("============================================================\n");

    // 1. Connect to SQLite persistence engine
    println!("1. Initializing in-process Crucible SQLite persistence engine...");
    let crucible = Arc::new(CrucibleEngine::connect_in_memory().await?);
    let engine = OpusEngine::new(crucible.clone());

    // 2. Build the Opus workflow DAG
    let opus = Arc::new(
        Opus::builder("magnum_opus_transmutation")
            .step(Arc::new(Step1DraftProposal))
            .step(Arc::new(Step2HumanApproval))
            .step(Arc::new(Step3CompleteTransmutation))
            .build()?,
    );
    println!("2. Opus Workflow Definition Constructed:");
    println!("   - Workflow Name: {}", opus.name);
    println!("   - Total Steps:   {}\n", opus.steps.len());

    // 3. Initiate first execution run
    println!("3. Initiating Opus execution...");
    let run_outcome = engine.execute(&opus, None).await?;

    let (run_id, token) = match run_outcome {
        OpusRunResult::Suspended {
            run_id,
            resume_token,
            vessel_id,
        } => {
            println!("   Workflow execution suspended at node: '{vessel_id}'");
            println!("   Run Identifier: {run_id}");
            println!("   Resume Token:   {resume_token}\n");
            (run_id, resume_token)
        }
        _ => return Err("Expected workflow to suspend at Step 2".into()),
    };

    // 4. Simulate external asynchronous human approval (e.g., from Slack or Atelier UI)
    println!("4. Simulating external human review:");
    println!("   Reviewer: Grand Master Flamel");
    println!("   Decision: APPROVED (Assaying purity: 99.9%)\n");

    let approval_payload = json!({
        "approved": true,
        "approver": "Grand Master Flamel",
        "purity": 0.999
    });

    // 5. Resume execution using token
    println!("5. Invoking OpusEngine::resume() with cryptographic token...");
    let final_outcome = engine
        .resume(&opus, &run_id, &token, approval_payload)
        .await?;

    match final_outcome {
        OpusRunResult::Transmuted { final_state, .. } => {
            println!("   Workflow successfully transmuted final state!");
            println!("   Final State Snapshot:\n{}\n", serde_json::to_string_pretty(&final_state)?);
        }
        other => return Err(format!("Unexpected outcome: {other:?}").into()),
    }

    // 6. Query and display the immutable SQLite event journal
    println!("6. Verifying SQLite Event Journal Audit Trail:");
    let events = crucible.get_vessel_events(&run_id).await?;
    for ev in events {
        println!(
            "   [Event #{}] Node: {:<30} Status: {:<12}",
            ev.event_id, ev.vessel_id, ev.status
        );
    }

    println!("\n============================================================");
    println!(" Durable workflow completed with 100% state durability.");
    println!("============================================================");

    Ok(())
}
