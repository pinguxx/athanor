//! Error types for the Atelier Dev Studio visualizer and observability server.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

/// Atelier server, registry, or telemetry errors.
#[derive(Debug, Error)]
pub enum AtelierError {
    /// Server initialization or runtime failure.
    #[error("Server error: {0}")]
    Server(String),

    /// Component registry failure.
    #[error("Registry error: {0}")]
    Registry(String),

    /// Workflow resumption error.
    #[error("Resume error: {0}")]
    Resume(String),

    /// Telemetry broadcasting error.
    #[error("Telemetry error: {0}")]
    Telemetry(String),

    /// Serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Underlying Opus workflow engine error.
    #[error("Opus error: {0}")]
    Opus(#[from] athanor_opus::OpusError),

    /// Underlying Crucible persistence error.
    #[error("Crucible error: {0}")]
    Crucible(#[from] athanor_crucible::CrucibleError),
}

impl From<serde_json::Error> for AtelierError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl IntoResponse for AtelierError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::Registry(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Resume(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Serialization(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Opus(err) => match err {
                athanor_opus::OpusError::Resume(_) => (StatusCode::BAD_REQUEST, err.to_string()),
                athanor_opus::OpusError::Crucible(
                    athanor_crucible::CrucibleError::TokenNotFound(_),
                ) => (StatusCode::BAD_REQUEST, err.to_string()),
                athanor_opus::OpusError::Crucible(
                    athanor_crucible::CrucibleError::RunNotFound(_),
                ) => (StatusCode::NOT_FOUND, err.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            },
            Self::Crucible(err) => match err {
                athanor_crucible::CrucibleError::TokenNotFound(_) => {
                    (StatusCode::BAD_REQUEST, err.to_string())
                }
                athanor_crucible::CrucibleError::RunNotFound(_) => {
                    (StatusCode::NOT_FOUND, err.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = json!({
            "error": msg,
            "status": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}
