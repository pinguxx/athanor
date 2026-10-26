//! Embedded static assets for the Atelier Dev Studio visualizer.

use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// Pre-compiled static assets embedded directly into the binary from `ui/dist/`.
#[derive(RustEmbed)]
#[folder = "ui/dist/"]
pub struct StaticAssets;

/// Serve static embedded assets with fallback to `index.html` for single-page applications.
pub async fn static_asset_handler(uri: axum::http::Uri) -> Response {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = mime_type_for_path(&path);
            let mut response = Response::new(Body::from(content.data));
            if let Ok(val) = HeaderValue::from_str(mime) {
                response.headers_mut().insert(header::CONTENT_TYPE, val);
            }
            response
        }
        None => {
            // Single Page Application (SPA) fallback to index.html
            match StaticAssets::get("index.html") {
                Some(content) => {
                    let mut response = Response::new(Body::from(content.data));
                    if let Ok(val) = HeaderValue::from_str("text/html; charset=utf-8") {
                        response.headers_mut().insert(header::CONTENT_TYPE, val);
                    }
                    response
                }
                None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            }
        }
    }
}

/// Determine MIME type from file extension.
pub fn mime_type_for_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else {
        "application/octet-stream"
    }
}
