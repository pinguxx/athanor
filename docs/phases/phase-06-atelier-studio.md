# Phase 6: Atelier Dev Studio & Observability Server

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-atelier`
- **Primary Goal**: Deliver a single-binary developer observatory and visualizer. An embedded Axum web server broadcasts real-time execution traces over Server-Sent Events (SSE), serves pre-compiled Svelte frontend assets directly from memory using `rust-embed`, and allows developers to inspect historical step checkpoints and trigger workflow resumptions.

---

## 2. Detailed Task Breakdown

### Task 6.1: Scaffold `rubedo-atelier`
- Create `crates/rubedo-atelier/Cargo.toml` with dependencies:
  - `axum = "0.7"`
  - `tower-http = { version = "0.5", features = ["cors", "trace"] }`
  - `rust-embed = "8.5"`
  - `rubedo-core`, `rubedo-opus`, `rubedo-crucible`
  - `tokio = { version = "1.40", features = ["full"] }`
  - `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber`

### Task 6.2: Telemetry Pipeline & SSE Broadcaster
- In `crates/rubedo-atelier/src/telemetry.rs`:
  - Create `AtelierEvent` enum (`VesselStarted`, `VesselCompleted`, `VesselSuspended`, `TokenUsage`, `LogMessage`).
  - Implement a Tokio broadcast channel (`tokio::sync::broadcast::Sender<AtelierEvent>`).
  - Implement a custom `tracing_subscriber::Layer` that captures Rubedo spans and dispatches `AtelierEvent` items to the broadcast channel.

### Task 6.3: HTTP & SSE API Endpoints
- In `crates/rubedo-atelier/src/server.rs`:
  - Define Axum router:
    ```rust
    let app = Router::new()
        .route("/api/registry", get(get_registry))
        .route("/api/events", get(sse_handler))
        .route("/api/opus/:run_id/resume", post(resume_opus))
        .fallback(static_asset_handler);
    ```
  - `get_registry`: returns JSON listing of all registered Artificers, Opuses, and Reagents.
  - `sse_handler`: subscribes to the broadcast channel and streams events formatted as `text/event-stream`.
  - `resume_opus`: accepts `{ "token": "...", "payload": {...} }` and invokes `OpusEngine::resume()`.

### Task 6.4: Embedded Static Asset Serving
- In `crates/rubedo-atelier/src/assets.rs`:
  - Define `#[derive(RustEmbed)]` struct pointing to `ui/dist` (or fallback embedded HTML).
  - Serve `index.html`, JavaScript, and CSS bundles directly from binary memory with appropriate MIME types.

---

## 3. Verification & Acceptance Criteria

### Integration Test
Create `crates/rubedo-atelier/tests/atelier_server_test.rs`:
1. Initialize `AtelierServer` bound to `127.0.0.1:0` (ephemeral port).
2. Spawn server task in Tokio background.
3. Perform HTTP `GET /api/registry`:
   - Assert HTTP status is `200 OK`.
   - Assert JSON response contains registered components.
4. Perform HTTP `GET /api/events`:
   - Assert `Content-Type` is `text/event-stream`.
5. Shut down server cleanly.

### Execution Command
```bash
cargo test -p rubedo-atelier
```
