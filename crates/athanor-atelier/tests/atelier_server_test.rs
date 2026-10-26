use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::sync::oneshot;

use athanor_atelier::prelude::*;
use athanor_core::{Artificer, Reagent, ReagentError};
use athanor_crucible::CrucibleEngine;
use athanor_opus::prelude::*;

// ---------------------------------------------------------------------------
// Mock Reagent for Testing
// ---------------------------------------------------------------------------

struct EchoReagent;

#[async_trait]
impl Reagent for EchoReagent {
    fn name(&self) -> &'static str {
        "echo_reagent"
    }

    fn description(&self) -> &'static str {
        "Echoes input text back to the caller"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" }
            },
            "required": ["text"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "echo": { "type": "string" }
            }
        })
    }

    async fn transmute(&self, input: Value) -> Result<Value, ReagentError> {
        let text = input
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        Ok(json!({ "echo": text }))
    }
}

// ---------------------------------------------------------------------------
// Mock Vessels for Workflow Testing
// ---------------------------------------------------------------------------

struct Step1Init;

#[async_trait]
impl Vessel for Step1Init {
    fn id(&self) -> &'static str {
        "step_1_init"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        ctx.set("val", 42)
            .map_err(|e| VesselError::Execution(e.to_string()))?;
        Ok(VesselStatus::Transmuted {
            output: json!({ "initialized": true, "val": 42 }),
        })
    }
}

struct Step2Suspend;

#[async_trait]
impl Vessel for Step2Suspend {
    fn id(&self) -> &'static str {
        "step_2_suspend"
    }

    async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
        let val: i64 = ctx
            .get("val")
            .ok_or_else(|| VesselError::InvalidState("Missing 'val'".into()))?;

        if val == 42 {
            let token = format!("token-approval-{}", uuid::Uuid::new_v4());
            Ok(VesselStatus::Suspended {
                resume_token: token,
            })
        } else {
            Ok(VesselStatus::Transmuted {
                output: json!({ "skipped": true }),
            })
        }
    }
}

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

        let bonus = ctx
            .resume_payload()
            .and_then(|p| p.get("bonus"))
            .and_then(|b| b.as_i64())
            .unwrap_or(0);

        let final_sum = val + bonus;
        ctx.set("final_sum", final_sum)
            .map_err(|e| VesselError::Execution(e.to_string()))?;

        Ok(VesselStatus::Transmuted {
            output: json!({ "final_sum": final_sum }),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_server_startup_and_registry_api() {
    let registry = AtelierRegistry::new();
    let broadcaster = TelemetryBroadcaster::new(64);

    // Register an Artificer
    let mut artificer = Artificer::builder()
        .name("HermeticScholar")
        .instructions("Conduct chemical inquiries.")
        .model("claude-3-5-sonnet")
        .max_steps(5)
        .build()
        .expect("build artificer");
    artificer.add_reagent(Arc::new(EchoReagent));
    registry.register_artificer(&artificer);

    // Register an Opus
    let opus = Opus::builder("hermetic_cycle")
        .step(Arc::new(Step1Init))
        .build()
        .expect("build opus");
    registry.register_opus(Arc::new(opus));

    // Register a Reagent
    let reagent = EchoReagent;
    registry.register_reagent(&reagent);

    // Bind server to ephemeral port
    let server = AtelierServer::new(registry, broadcaster);
    let bound = server
        .bind("127.0.0.1:0")
        .await
        .expect("bind atelier server");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("server run");
    });

    // Make HTTP GET request to /api/registry
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/api/registry"))
        .send()
        .await
        .expect("GET /api/registry");

    assert_eq!(resp.status(), StatusCode::OK);

    let reg_response: RegistryResponse = resp.json().await.expect("deserialize registry");
    assert_eq!(reg_response.artificers.len(), 1);
    assert_eq!(reg_response.artificers[0].name, "HermeticScholar");
    assert_eq!(reg_response.artificers[0].model, "claude-3-5-sonnet");
    assert_eq!(
        reg_response.artificers[0].reagents,
        vec!["echo_reagent".to_string()]
    );

    assert_eq!(reg_response.opuses.len(), 1);
    assert_eq!(reg_response.opuses[0].name, "hermetic_cycle");
    assert_eq!(reg_response.opuses[0].step_ids, vec!["step_1_init"]);

    assert_eq!(reg_response.reagents.len(), 1);
    assert_eq!(reg_response.reagents[0].name, "echo_reagent");

    // Clean shutdown
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_server_static_assets_and_spa_fallback() {
    let registry = AtelierRegistry::new();
    let broadcaster = TelemetryBroadcaster::new(64);
    let server = AtelierServer::new(registry, broadcaster);
    let bound = server.bind("127.0.0.1:0").await.expect("bind server");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("server run");
    });

    let client = reqwest::Client::new();

    // 1. GET /
    let resp = client
        .get(format!("http://{addr}/"))
        .send()
        .await
        .expect("GET /");
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .expect("content type header")
        .to_str()
        .expect("valid str");
    assert!(content_type.contains("text/html"));
    let body = resp.text().await.expect("body text");
    assert!(body.contains("Athanor Atelier"));

    // 2. GET /index.html
    let resp = client
        .get(format!("http://{addr}/index.html"))
        .send()
        .await
        .expect("GET /index.html");
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. SPA Fallback: GET /dashboard/custom-view
    let resp = client
        .get(format!("http://{addr}/dashboard/custom-view"))
        .send()
        .await
        .expect("GET /dashboard/custom-view");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.expect("fallback body");
    assert!(body.contains("Athanor Atelier"));

    // Shutdown
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_server_sse_event_streaming() {
    let registry = AtelierRegistry::new();
    let broadcaster = TelemetryBroadcaster::new(128);
    let server = AtelierServer::new(registry, broadcaster.clone());
    let bound = server.bind("127.0.0.1:0").await.expect("bind server");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("server run");
    });

    let client = reqwest::Client::new();
    let mut resp = client
        .get(format!("http://{addr}/api/events"))
        .send()
        .await
        .expect("GET /api/events");

    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .expect("content type")
        .to_str()
        .expect("str");
    assert!(content_type.contains("text/event-stream"));

    // Give a tiny yield to ensure SSE stream is active
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Broadcast an event
    let event = AtelierEvent::vessel_started("run-test-42", "step_init");
    broadcaster.broadcast(event).expect("broadcast event");

    // Read first chunk from the SSE stream
    let chunk_timeout = tokio::time::timeout(Duration::from_secs(3), resp.chunk()).await;
    let chunk = chunk_timeout
        .expect("timeout waiting for SSE chunk")
        .expect("chunk read error")
        .expect("expected non-empty chunk");

    let chunk_str = String::from_utf8_lossy(&chunk);
    assert!(chunk_str.contains("run-test-42"));
    assert!(chunk_str.contains("step_init"));

    // Drop open SSE stream to allow server graceful shutdown
    drop(resp);
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_server_opus_resumption_api() {
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory db"),
    );
    let opus_engine = Arc::new(OpusEngine::new(crucible.clone()));

    let opus = Arc::new(
        Opus::builder("durable_resumption_workflow")
            .step(Arc::new(Step1Init))
            .step(Arc::new(Step2Suspend))
            .step(Arc::new(Step3Finalize))
            .build()
            .expect("build opus"),
    );

    // Initial execution suspends at Step 2
    let run_res = opus_engine
        .execute(&opus, None)
        .await
        .expect("initial execute");
    let (run_id, resume_token) = match &run_res {
        OpusRunResult::Suspended {
            run_id,
            resume_token,
            ..
        } => (run_id.clone(), resume_token.clone()),
        other => panic!("Expected Suspended result, got {other:?}"),
    };

    let registry = AtelierRegistry::new();
    registry.register_opus(opus);

    let broadcaster = TelemetryBroadcaster::new(64);
    let server = AtelierServer::new(registry, broadcaster)
        .with_crucible(crucible)
        .with_opus_engine(opus_engine);

    let bound = server.bind("127.0.0.1:0").await.expect("bind server");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("server run");
    });

    let client = reqwest::Client::new();

    // 1. Invalid resume token should fail with 400 Bad Request
    let bad_resp = client
        .post(format!("http://{addr}/api/opus/{run_id}/resume"))
        .json(&json!({
            "token": "INVALID_TOKEN_XYZ",
            "payload": { "bonus": 58 }
        }))
        .send()
        .await
        .expect("POST bad resume");
    assert_eq!(bad_resp.status(), StatusCode::BAD_REQUEST);

    // 2. Valid resume token should resume and transmute final state
    let good_resp = client
        .post(format!("http://{addr}/api/opus/{run_id}/resume"))
        .json(&json!({
            "token": resume_token,
            "payload": { "bonus": 58 }
        }))
        .send()
        .await
        .expect("POST good resume");
    assert_eq!(good_resp.status(), StatusCode::OK);

    let outcome: OpusRunResult = good_resp.json().await.expect("deserialize outcome");
    match outcome {
        OpusRunResult::Transmuted {
            final_state,
            run_id: resumed_id,
        } => {
            assert_eq!(resumed_id, run_id);
            assert_eq!(
                final_state.get("final_sum").and_then(|v| v.as_i64()),
                Some(100)
            );
        }
        other => panic!("Expected Transmuted outcome, got {other:?}"),
    }

    // Shutdown
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_telemetry_broadcaster_and_tracing_layer() {
    use tracing_subscriber::layer::SubscriberExt;

    let broadcaster = TelemetryBroadcaster::new(32);
    let mut rx = broadcaster.subscribe();

    let layer = broadcaster.tracing_layer();
    let subscriber = tracing_subscriber::Registry::default().with(layer);

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Telemetry tracing integration verified");
    });

    let received = rx.try_recv().expect("received tracing event");
    match received {
        AtelierEvent::LogMessage { level, message, .. } => {
            assert_eq!(level, "INFO");
            assert!(message.contains("Telemetry tracing integration verified"));
        }
        other => panic!("Expected LogMessage event, got {other:?}"),
    }
}

#[test]
fn test_mime_type_resolution() {
    use athanor_atelier::assets::mime_type_for_path;

    assert_eq!(mime_type_for_path("index.html"), "text/html; charset=utf-8");
    assert_eq!(
        mime_type_for_path("bundle.js"),
        "application/javascript; charset=utf-8"
    );
    assert_eq!(mime_type_for_path("style.css"), "text/css; charset=utf-8");
    assert_eq!(mime_type_for_path("data.json"), "application/json");
    assert_eq!(mime_type_for_path("logo.svg"), "image/svg+xml");
    assert_eq!(mime_type_for_path("icon.png"), "image/png");
    assert_eq!(mime_type_for_path("favicon.ico"), "image/x-icon");
    assert_eq!(mime_type_for_path("module.wasm"), "application/wasm");
    assert_eq!(mime_type_for_path("data.bin"), "application/octet-stream");
}

#[tokio::test]
async fn test_atelier_error_responses() {
    use athanor_crucible::CrucibleError;
    use athanor_opus::OpusError;
    use axum::response::IntoResponse;

    // Registry error -> 404
    let err = AtelierError::Registry("component missing".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Resume error -> 400
    let err = AtelierError::Resume("invalid token".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Serialization error -> 400
    let err = AtelierError::Serialization("bad json".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // From serde_json::Error -> Serialization -> 400
    let serde_err = serde_json::from_str::<Value>("not json").unwrap_err();
    let err: AtelierError = serde_err.into();
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Opus(TokenNotFound) -> 400
    let err = AtelierError::Opus(OpusError::Crucible(CrucibleError::TokenNotFound(
        "tok".into(),
    )));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Opus(RunNotFound) -> 404
    let err = AtelierError::Opus(OpusError::Crucible(CrucibleError::RunNotFound(
        "run".into(),
    )));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Opus(Resume) -> 400
    let err = AtelierError::Opus(OpusError::Resume("bad state".into()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Opus(Topology) -> 500
    let err = AtelierError::Opus(OpusError::Topology("topology error".into()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Crucible(TokenNotFound) -> 400
    let err = AtelierError::Crucible(CrucibleError::TokenNotFound("tok".into()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Crucible(RunNotFound) -> 404
    let err = AtelierError::Crucible(CrucibleError::RunNotFound("run".into()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Crucible(Migration) -> 500
    let err = AtelierError::Crucible(CrucibleError::Migration("migration failure".into()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Server error -> 500
    let err = AtelierError::Server("listen error".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Telemetry error -> 500
    let err = AtelierError::Telemetry("channel full".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_atelier_events_and_telemetry_broadcaster_helpers() {
    let completed = AtelierEvent::vessel_completed("run-1", "step-1", "Transmuted");
    match completed {
        AtelierEvent::VesselCompleted { run_id, status, .. } => {
            assert_eq!(run_id, "run-1");
            assert_eq!(status, "Transmuted");
        }
        _ => panic!("wrong variant"),
    }

    let suspended = AtelierEvent::vessel_suspended("run-2", "step-2", "tok-abc");
    match suspended {
        AtelierEvent::VesselSuspended { resume_token, .. } => {
            assert_eq!(resume_token, "tok-abc");
        }
        _ => panic!("wrong variant"),
    }

    let usage = AtelierEvent::token_usage("ArtificerX", 120, 80);
    match usage {
        AtelierEvent::TokenUsage {
            total_tokens,
            prompt_tokens,
            completion_tokens,
            ..
        } => {
            assert_eq!(prompt_tokens, 120);
            assert_eq!(completion_tokens, 80);
            assert_eq!(total_tokens, 200);
        }
        _ => panic!("wrong variant"),
    }

    let log = AtelierEvent::log_message("WARN", "athanor::test", "warning text");
    match log {
        AtelierEvent::LogMessage { level, message, .. } => {
            assert_eq!(level, "WARN");
            assert_eq!(message, "warning text");
        }
        _ => panic!("wrong variant"),
    }

    let broadcaster = TelemetryBroadcaster::default();
    assert_eq!(broadcaster.subscriber_count(), 0);
    let _sub = broadcaster.subscribe();
    assert_eq!(broadcaster.subscriber_count(), 1);
}

#[tokio::test]
async fn test_server_resume_error_paths() {
    let registry = AtelierRegistry::new();
    let broadcaster = TelemetryBroadcaster::new(16);

    // 1. Server missing OpusEngine and Crucible
    let server = AtelierServer::new(registry.clone(), broadcaster.clone());
    let bound = server.bind("127.0.0.1:0").await.expect("bind");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("run");
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/api/opus/some-run/resume"))
        .json(&json!({ "token": "any" }))
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let _ = shutdown_tx.send(());
    let _ = server_handle.await;

    // 2. Server with OpusEngine but no Crucible configured
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory db"),
    );
    let opus_engine = Arc::new(OpusEngine::new(crucible.clone()));
    let server_no_crucible = AtelierServer::new(registry.clone(), broadcaster.clone())
        .with_opus_engine(opus_engine.clone());
    let bound = server_no_crucible.bind("127.0.0.1:0").await.expect("bind");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("run");
    });

    let resp = client
        .post(format!("http://{addr}/api/opus/some-run/resume"))
        .json(&json!({ "token": "any" }))
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let _ = shutdown_tx.send(());
    let _ = server_handle.await;

    // 3. Server with Crucible and OpusEngine, but run not found in DB
    let server_full = AtelierServer::new(registry.clone(), broadcaster.clone())
        .with_crucible(crucible.clone())
        .with_opus_engine(opus_engine.clone());

    // Check getters
    assert_eq!(server_full.broadcaster().subscriber_count(), 0);
    let _ = server_full.registry();
    let _ = server_full.router();

    let bound = server_full.bind("127.0.0.1:0").await.expect("bind");
    let addr = bound.local_addr();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        bound
            .run_with_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("run");
    });

    let resp = client
        .post(format!("http://{addr}/api/opus/non-existent-run/resume"))
        .json(&json!({ "token": "any" }))
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
}

#[test]
fn test_registry_summary_registration() {
    let registry = AtelierRegistry::new();

    registry.register_artificer_summary(ArtificerSummary {
        name: "DirectArtificer".to_string(),
        instructions: "Do math".to_string(),
        model: "gpt-4o".to_string(),
        max_steps: 10,
        reagents: vec!["calculator".to_string()],
    });

    registry.register_reagent_summary(ReagentSummary {
        name: "DirectReagent".to_string(),
        description: "Executes calculations".to_string(),
        input_schema: json!({ "type": "object" }),
        output_schema: json!({ "type": "number" }),
    });

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.artificers.len(), 1);
    assert_eq!(snapshot.artificers[0].name, "DirectArtificer");
    assert_eq!(snapshot.reagents.len(), 1);
    assert_eq!(snapshot.reagents[0].name, "DirectReagent");
}
