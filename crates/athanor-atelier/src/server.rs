//! HTTP and Server-Sent Events (SSE) router and server implementation.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use athanor_crucible::CrucibleEngine;
use athanor_opus::{OpusEngine, OpusRunResult};

use crate::assets::static_asset_handler;
use crate::error::AtelierError;
use crate::registry::{AtelierRegistry, RegistryResponse};
use crate::telemetry::{AtelierEvent, TelemetryBroadcaster};

/// Shared state container accessible across all Atelier HTTP request handlers.
#[derive(Clone)]
pub struct ServerState {
    /// In-memory component registry.
    pub registry: AtelierRegistry,
    /// Observability and trace event broadcaster.
    pub broadcaster: TelemetryBroadcaster,
    /// Optional Crucible SQLite storage engine.
    pub crucible: Option<Arc<CrucibleEngine>>,
    /// Optional Opus state machine engine for resumption.
    pub opus_engine: Option<Arc<OpusEngine>>,
}

/// Request body schema for resuming a suspended Opus workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeRequestBody {
    /// Cryptographic resume token obtained upon workflow suspension.
    pub token: String,
    /// Arbitrary external JSON payload to inject into the resumed workflow context.
    #[serde(default)]
    pub payload: Option<Value>,
}

/// Atelier Dev Studio HTTP server.
#[derive(Clone)]
pub struct AtelierServer {
    state: ServerState,
}

impl AtelierServer {
    /// Construct a new Atelier server instance with registry and telemetry broadcaster.
    pub fn new(registry: AtelierRegistry, broadcaster: TelemetryBroadcaster) -> Self {
        Self {
            state: ServerState {
                registry,
                broadcaster,
                crucible: None,
                opus_engine: None,
            },
        }
    }

    /// Attach a Crucible persistence engine for querying workflow runs.
    pub fn with_crucible(mut self, crucible: Arc<CrucibleEngine>) -> Self {
        self.state.crucible = Some(crucible);
        self
    }

    /// Attach an Opus engine for workflow resumption.
    pub fn with_opus_engine(mut self, opus_engine: Arc<OpusEngine>) -> Self {
        self.state.opus_engine = Some(opus_engine);
        self
    }

    /// Return a reference to the component registry.
    pub fn registry(&self) -> &AtelierRegistry {
        &self.state.registry
    }

    /// Return a reference to the telemetry broadcaster.
    pub fn broadcaster(&self) -> &TelemetryBroadcaster {
        &self.state.broadcaster
    }

    /// Build the configured Axum router.
    pub fn router(&self) -> Router {
        build_router(Arc::new(self.state.clone()))
    }

    /// Bind the server to the given network address.
    pub async fn bind(
        &self,
        addr: impl tokio::net::ToSocketAddrs,
    ) -> Result<BoundAtelierServer, AtelierError> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        let router = build_router(Arc::new(self.state.clone()));
        Ok(BoundAtelierServer {
            listener,
            local_addr,
            router,
        })
    }
}

/// A running Atelier HTTP server bound to a local TCP socket.
pub struct BoundAtelierServer {
    listener: tokio::net::TcpListener,
    local_addr: SocketAddr,
    router: Router,
}

impl BoundAtelierServer {
    /// Return the bound socket address (including assigned ephemeral port).
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Run the server until process termination.
    pub async fn run(self) -> Result<(), AtelierError> {
        axum::serve(self.listener, self.router)
            .await
            .map_err(|e| AtelierError::Server(e.to_string()))
    }

    /// Run the server with graceful shutdown when the provided signal resolves.
    pub async fn run_with_shutdown<F>(self, signal: F) -> Result<(), AtelierError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(signal)
            .await
            .map_err(|e| AtelierError::Server(e.to_string()))
    }
}

/// Construct the Axum router for the Atelier Dev Studio.
pub fn build_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/api/registry", get(get_registry))
        .route("/api/events", get(sse_handler))
        .route("/api/opus/:run_id/resume", post(resume_opus))
        .fallback(static_asset_handler)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Handler returning a snapshot of all registered components.
async fn get_registry(State(state): State<Arc<ServerState>>) -> Json<RegistryResponse> {
    Json(state.registry.snapshot())
}

/// Handler streaming real-time execution events via Server-Sent Events (SSE).
async fn sse_handler(
    State(state): State<Arc<ServerState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.broadcaster.subscribe();

    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => match serde_json::to_string(&event) {
                    Ok(json) => {
                        let sse_event = Event::default().data(json);
                        return Some((Ok(sse_event), rx));
                    }
                    Err(_) => continue,
                },
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Handler resuming a suspended Opus workflow given a valid resumption token.
async fn resume_opus(
    State(state): State<Arc<ServerState>>,
    Path(run_id): Path<String>,
    Json(body): Json<ResumeRequestBody>,
) -> Result<Json<OpusRunResult>, AtelierError> {
    let engine = state.opus_engine.as_ref().ok_or_else(|| {
        AtelierError::Resume("OpusEngine is not configured on this server".to_string())
    })?;

    let crucible = state.crucible.as_ref().ok_or_else(|| {
        AtelierError::Resume("CrucibleEngine is not configured on this server".to_string())
    })?;

    let record = crucible.get_run(&run_id).await?;
    let opus = state.registry.get_opus(&record.opus_name).ok_or_else(|| {
        AtelierError::Registry(format!(
            "Opus workflow '{}' for run '{}' is not registered in Atelier",
            record.opus_name, run_id
        ))
    })?;

    let payload = match body.payload {
        Some(p) => p,
        None => Value::Null,
    };

    let result = engine.resume(&opus, &run_id, &body.token, payload).await?;

    // Emit telemetry event based on outcome
    match &result {
        OpusRunResult::Transmuted { .. } => {
            let _ = state.broadcaster.broadcast(AtelierEvent::vessel_completed(
                &run_id,
                "Workflow",
                "Transmuted",
            ));
        }
        OpusRunResult::Suspended {
            vessel_id,
            resume_token,
            ..
        } => {
            let _ = state.broadcaster.broadcast(AtelierEvent::vessel_suspended(
                &run_id,
                vessel_id,
                resume_token,
            ));
        }
        OpusRunResult::Corrupted { vessel_id, .. } => {
            let _ = state.broadcaster.broadcast(AtelierEvent::vessel_completed(
                &run_id,
                vessel_id,
                "Corrupted",
            ));
        }
    }

    Ok(Json(result))
}
