use async_trait::async_trait;
use athanor_crucible::CrucibleEngine;
use athanor_opus::prelude::*;
use serde_json::json;
use std::sync::Arc;

/// Step 1: Computes initial value
struct Step1Initialize;

#[async_trait]
impl Vessel for Step1Initialize {
    fn id(&self) -> &'static str {
        "step_1_init"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        ctx.set("stage", 1)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("val", 42)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        Ok(VesselStatus::Transmuted {
            output: json!({ "status": "initialized", "val": 42 }),
        })
    }
}

/// Step 2: Suspends execution and emits an unforgeable resume token
struct Step2Suspend;

#[async_trait]
impl Vessel for Step2Suspend {
    fn id(&self) -> &'static str {
        "step_2_suspend"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let val: i64 = ctx
            .get("val")
            .ok_or_else(|| VesselError::InvalidState("Missing 'val' in state".into()))?;

        if val == 42 {
            let token = format!("token-approval-{}", uuid::Uuid::new_v4());
            ctx.set("stage", 2)
                .map_err(|e| VesselError::Execution(e.to_string()))?;
            Ok(VesselStatus::Suspended {
                resume_token: token,
            })
        } else {
            Ok(VesselStatus::Transmuted {
                output: json!({ "skipped_suspension": true }),
            })
        }
    }
}

/// Step 3: Resumes with external payload and completes computation
struct Step3Finalize;

#[async_trait]
impl Vessel for Step3Finalize {
    fn id(&self) -> &'static str {
        "step_3_finalize"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let val: i64 = ctx
            .get("val")
            .ok_or_else(|| VesselError::InvalidState("Missing 'val'".into()))?;

        // Read the injected resume payload
        let bonus = ctx
            .resume_payload()
            .and_then(|p| p.get("bonus"))
            .and_then(|b| b.as_i64())
            .unwrap_or(0);

        let final_sum = val + bonus;
        ctx.set("stage", 3)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        ctx.set("final_sum", final_sum)
            .map_err(|e| VesselError::Execution(e.to_string()))?;

        Ok(VesselStatus::Transmuted {
            output: json!({ "final_sum": final_sum }),
        })
    }
}

#[tokio::test]
async fn test_durable_workflow_suspension_and_resumption() {
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory db"),
    );
    let engine = OpusEngine::new(crucible.clone());

    let opus = Opus::builder("transmutation_magnum_opus")
        .step(Arc::new(Step1Initialize))
        .step(Arc::new(Step2Suspend))
        .step(Arc::new(Step3Finalize))
        .build()
        .expect("valid opus");

    // 1. Initial execution halts at Step 2
    let result = engine.execute(&opus, None).await.expect("execute");
    assert!(result.is_suspended());

    let (run_id, resume_token) = match &result {
        OpusRunResult::Suspended {
            run_id,
            vessel_id,
            resume_token,
        } => {
            assert_eq!(vessel_id, "step_2_suspend");
            assert!(resume_token.starts_with("token-approval-"));
            (run_id.clone(), resume_token.clone())
        }
        _ => panic!("Expected Suspended result"),
    };

    // 2. Verify state is committed to SQLite in Crucible
    let db_run = crucible.get_run(&run_id).await.expect("get run from db");
    assert_eq!(db_run.status, "Suspended");
    assert_eq!(db_run.current_state["val"], 42);
    assert_eq!(db_run.current_state["stage"], 2);

    let token_record = crucible
        .get_suspended_token(&resume_token)
        .await
        .expect("get token");
    assert!(token_record.is_some());
    assert_eq!(token_record.unwrap().vessel_id, "step_2_suspend");

    // 3. Resume the workflow with external payload: {"bonus": 10}
    let resume_payload = json!({ "bonus": 10 });
    let resume_result = engine
        .resume(&opus, &run_id, &resume_token, resume_payload)
        .await
        .expect("resume");

    assert!(resume_result.is_transmuted());
    match resume_result {
        OpusRunResult::Transmuted {
            run_id: final_run_id,
            final_state,
        } => {
            assert_eq!(final_run_id, run_id);
            assert_eq!(final_state["val"], 42);
            assert_eq!(final_state["stage"], 3);
            assert_eq!(final_state["final_sum"], 52);
        }
        _ => panic!("Expected Transmuted result"),
    }

    // 4. Verify SQLite state after completion
    let completed_run = crucible.get_run(&run_id).await.expect("completed run");
    assert_eq!(completed_run.status, "Completed");
    assert_eq!(completed_run.current_state["final_sum"], 52);

    // 5. Verify resume token was consumed from SQLite
    let token_after = crucible
        .get_suspended_token(&resume_token)
        .await
        .expect("token after");
    assert!(token_after.is_none());

    // 6. Attempting to resume again fails with TokenNotFound
    let replay_attempt = engine
        .resume(&opus, &run_id, &resume_token, json!({ "bonus": 20 }))
        .await;
    assert!(replay_attempt.is_err());

    // 7. Verify complete event audit trail
    let events = crucible.get_vessel_events(&run_id).await.expect("events");
    assert!(events.len() >= 5);
}

#[tokio::test]
async fn test_opus_builder_topology_validation() {
    // Empty steps rejected
    let empty_err = Opus::builder("empty_opus").build().unwrap_err();
    assert!(empty_err.to_string().contains("at least one Vessel step"));

    // Duplicate step IDs rejected
    let dup_err = Opus::builder("duplicate_opus")
        .step(Arc::new(Step1Initialize))
        .step(Arc::new(Step1Initialize))
        .build()
        .unwrap_err();
    assert!(dup_err
        .to_string()
        .contains("Duplicate Vessel step identifier"));

    // Multiple steps via .steps() builder method and initial_state
    let builder = Opus::builder("multi_step")
        .steps(vec![Arc::new(Step1Initialize) as Arc<dyn Vessel>])
        .initial_state(json!({ "preloaded": true }));
    let debug_builder_str = format!("{builder:?}");
    assert!(debug_builder_str.contains("multi_step"));

    let opus = builder.build().expect("valid opus");
    let debug_opus_str = format!("{opus:?}");
    assert!(debug_opus_str.contains("step_1_init"));
    assert_eq!(opus.initial_state["preloaded"], true);
}

/// Failing step returning Err(VesselError::Execution(...))
struct FailingVessel;

#[async_trait]
impl Vessel for FailingVessel {
    fn id(&self) -> &'static str {
        "step_failing"
    }

    async fn distill(&self, _ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        Err(VesselError::Execution("Critical vessel combustion".into()))
    }
}

/// Failing step returning Ok(VesselStatus::Corrupted)
struct CorruptingVessel;

#[async_trait]
impl Vessel for CorruptingVessel {
    fn id(&self) -> &'static str {
        "step_corrupting"
    }

    async fn distill(&self, _ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        Ok(VesselStatus::Corrupted {
            error: "State decomposition failed".into(),
        })
    }
}

#[tokio::test]
async fn test_failing_vessel_handling() {
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory db"),
    );
    let engine = OpusEngine::new(crucible.clone());

    // 1. Vessel that returns Err(VesselError)
    let opus_err = Opus::builder("failing_workflow")
        .step(Arc::new(FailingVessel))
        .build()
        .expect("valid opus");

    let res = engine
        .execute(&opus_err, Some("run-failing-1"))
        .await
        .expect("execute");
    assert!(res.is_corrupted());
    assert!(!res.is_transmuted());
    assert!(!res.is_suspended());
    assert_eq!(res.run_id(), "run-failing-1");

    let run = crucible.get_run("run-failing-1").await.expect("get run");
    assert_eq!(run.status, "Failed");

    // 2. Vessel that returns Ok(VesselStatus::Corrupted)
    let opus_corrupt = Opus::builder("corrupted_workflow")
        .step(Arc::new(CorruptingVessel))
        .build()
        .expect("valid opus");

    let res_c = engine
        .execute(&opus_corrupt, Some("run-corrupt-1"))
        .await
        .expect("execute");
    assert!(res_c.is_corrupted());
    assert_eq!(res_c.run_id(), "run-corrupt-1");

    let run_c = crucible.get_run("run-corrupt-1").await.expect("get run");
    assert_eq!(run_c.status, "Failed");
}

#[tokio::test]
async fn test_resume_error_mismatches() {
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory db"),
    );
    let engine = OpusEngine::new(crucible.clone());

    let opus = Opus::builder("suspend_opus")
        .step(Arc::new(Step1Initialize))
        .step(Arc::new(Step2Suspend))
        .build()
        .expect("valid opus");

    let result = engine.execute(&opus, None).await.expect("execute");
    let (run_id, resume_token) = match result {
        OpusRunResult::Suspended {
            run_id,
            resume_token,
            ..
        } => (run_id, resume_token),
        _ => panic!("Expected Suspended"),
    };

    // 1. Attempting resume with wrong run_id
    let mismatch_res = engine
        .resume(&opus, "wrong-run-id", &resume_token, json!({}))
        .await;
    assert!(mismatch_res.is_err());
    let err_str = mismatch_res.unwrap_err().to_string();
    assert!(err_str.contains("belongs to run"));

    // 2. Re-register token under a run but with a step not in the workflow DAG
    let token2 = "tok-orphan-123";
    crucible
        .register_suspended_token(token2, &run_id, "orphan_vessel_not_in_dag")
        .await
        .expect("register token");

    let orphan_res = engine.resume(&opus, &run_id, token2, json!({})).await;
    assert!(orphan_res.is_err());
    let err_str2 = orphan_res.unwrap_err().to_string();
    assert!(err_str2.contains("was not found in Opus"));
}

#[test]
fn test_opus_context_operations() {
    // Test context initialization with null
    let ctx_null = OpusContext::new("run-null", serde_json::Value::Null);
    assert_eq!(ctx_null.run_id(), "run-null");
    assert!(ctx_null.state().is_object());

    // Test context initialization with scalar
    let ctx_scalar = OpusContext::new("run-scalar", json!(100));
    assert_eq!(ctx_scalar.state()["value"], 100);

    // Test get / get_value / set / set_value / set_state
    let mut ctx = OpusContext::new("run-1", json!({ "count": 10 }));
    assert_eq!(ctx.get::<i64>("count"), Some(10));
    assert_eq!(ctx.get::<i64>("nonexistent"), None);
    assert_eq!(ctx.get_value("count"), Some(&json!(10)));

    ctx.set("name", "Hermes").expect("set string");
    assert_eq!(ctx.get::<String>("name"), Some("Hermes".into()));

    ctx.set_value("raw", json!({ "nested": true }));
    assert_eq!(ctx.get_value("raw"), Some(&json!({ "nested": true })));

    ctx.set_state(json!({ "reset": true }));
    assert_eq!(ctx.state()["reset"], true);

    // Test resume payload
    assert!(ctx.resume_payload().is_none());
    ctx.set_resume_payload(json!({ "granted": true }));
    assert_eq!(ctx.resume_payload(), Some(&json!({ "granted": true })));
}

#[test]
fn test_opus_error_and_vessel_error_display() {
    let v_exec = VesselError::Execution("fail".into());
    assert_eq!(format!("{v_exec}"), "Vessel execution failed: fail");

    let v_state = VesselError::InvalidState("bad state".into());
    assert_eq!(format!("{v_state}"), "Invalid state: bad state");

    let v_fatal = VesselError::Fatal("fatal err".into());
    assert_eq!(format!("{v_fatal}"), "Fatal vessel failure: fatal err");

    let o_top = OpusError::Topology("bad topo".into());
    assert_eq!(format!("{o_top}"), "Workflow topology error: bad topo");

    let o_res = OpusError::Resume("bad resume".into());
    assert_eq!(format!("{o_res}"), "Resume error: bad resume");
}
