use rubedo_crucible::{CrucibleEngine, CrucibleError};
use serde_json::json;

#[tokio::test]
async fn test_crucible_lifecycle_and_checkpointing() {
    let engine = CrucibleEngine::connect_in_memory()
        .await
        .expect("in-memory connection");

    let run_id = "test-run-001";
    let initial_state = json!({ "counter": 0, "status": "init" });

    // 1. Insert run
    engine
        .insert_run(run_id, "transmutation_flow", &initial_state)
        .await
        .expect("insert run");

    let run = engine.get_run(run_id).await.expect("get run");
    assert_eq!(run.run_id, run_id);
    assert_eq!(run.opus_name, "transmutation_flow");
    assert_eq!(run.status, "Pending");
    assert_eq!(run.current_state, initial_state);

    // 2. Update run state
    let updated_state = json!({ "counter": 1, "status": "step_1_done" });
    engine
        .update_run_state(run_id, "Active", &updated_state)
        .await
        .expect("update run state");

    let run_updated = engine.get_run(run_id).await.expect("get updated run");
    assert_eq!(run_updated.status, "Active");
    assert_eq!(run_updated.current_state["counter"], 1);

    // 3. Record vessel events
    let event1 = engine
        .record_vessel_event(run_id, "vessel_calc", "Active", None)
        .await
        .expect("record event 1");
    assert_eq!(event1, 1);

    let event2 = engine
        .record_vessel_event(
            run_id,
            "vessel_calc",
            "Transmuted",
            Some(&json!({ "result": 42 })),
        )
        .await
        .expect("record event 2");
    assert_eq!(event2, 2);

    let events = engine.get_vessel_events(run_id).await.expect("get events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].vessel_id, "vessel_calc");
    assert_eq!(events[0].status, "Active");
    assert_eq!(events[0].payload, None);

    assert_eq!(events[1].status, "Transmuted");
    assert_eq!(events[1].payload, Some(json!({ "result": 42 })));

    // 4. List runs
    let runs = engine.list_runs(10).await.expect("list runs");
    assert_eq!(runs.len(), 1);
}

#[tokio::test]
async fn test_suspended_token_atomic_consumption() {
    let engine = CrucibleEngine::connect_in_memory()
        .await
        .expect("in-memory connection");

    let run_id = "test-run-token";
    engine
        .insert_run(run_id, "approval_flow", &json!({}))
        .await
        .expect("insert run");

    let token = "uuid-token-xyz-123";
    let vessel_id = "vessel_human_approval";

    // Register token
    engine
        .register_suspended_token(token, run_id, vessel_id)
        .await
        .expect("register token");

    // Inspect token
    let inspected = engine
        .get_suspended_token(token)
        .await
        .expect("inspect token");
    assert!(inspected.is_some());
    assert_eq!(inspected.as_ref().unwrap().run_id, run_id);
    assert_eq!(inspected.as_ref().unwrap().vessel_id, vessel_id);

    // First consumption: succeeds
    let (c_run, c_vessel) = engine
        .consume_suspended_token(token)
        .await
        .expect("consume token");
    assert_eq!(c_run, run_id);
    assert_eq!(c_vessel, vessel_id);

    // Second consumption: fails with TokenNotFound
    let second_attempt = engine.consume_suspended_token(token).await;
    assert!(second_attempt.is_err());
    match second_attempt {
        Err(CrucibleError::TokenNotFound(t)) => assert_eq!(t, token),
        _ => panic!("Expected TokenNotFound error"),
    }

    // Token is no longer in suspended_tokens
    let after_consume = engine
        .get_suspended_token(token)
        .await
        .expect("inspect after consume");
    assert!(after_consume.is_none());
}

#[tokio::test]
async fn test_run_not_found_error() {
    let engine = CrucibleEngine::connect_in_memory()
        .await
        .expect("in-memory connection");

    let err = engine.get_run("nonexistent-run").await.unwrap_err();
    assert!(matches!(err, CrucibleError::RunNotFound(_)));

    let update_err = engine
        .update_run_state("nonexistent-run", "Active", &json!({}))
        .await
        .unwrap_err();
    assert!(matches!(update_err, CrucibleError::RunNotFound(_)));
}

#[tokio::test]
async fn test_file_database_connection() {
    let tmp_path = std::env::temp_dir().join(format!("rubedo_test_{}.db", uuid::Uuid::new_v4()));
    
    let engine = CrucibleEngine::connect_file(&tmp_path)
        .await
        .expect("file database connection");

    let run_id = "file-run-001";
    engine
        .insert_run(run_id, "file_flow", &json!({ "connected": true }))
        .await
        .expect("insert run");

    let run = engine.get_run(run_id).await.expect("get run");
    assert_eq!(run.run_id, run_id);
    assert_eq!(run.current_state["connected"], true);

    // Clean up temporary database file
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_crucible_error_display() {
    let err1 = CrucibleError::RunNotFound("r-123".into());
    assert_eq!(format!("{err1}"), "Run not found: r-123");

    let err2 = CrucibleError::TokenNotFound("tok-abc".into());
    assert_eq!(format!("{err2}"), "Suspended token not found or already consumed: tok-abc");

    let err3 = CrucibleError::Migration("failed syntax".into());
    assert_eq!(format!("{err3}"), "Migration error: failed syntax");

    let err4 = CrucibleError::Invariant("broken invariant".into());
    assert_eq!(format!("{err4}"), "Invariant error: broken invariant");
}
