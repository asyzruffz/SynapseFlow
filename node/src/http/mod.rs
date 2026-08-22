//! Loopback HTTP transport for the common local generation workflow.

mod error;
mod handlers;
mod models;

use axum::{extract::DefaultBodyLimit, routing::post, Router};

use crate::LocalNode;

/// Builds a loopback-safe API router backed by the shared local node workflow.
pub fn router(node: LocalNode) -> Router {
    Router::new()
        .route("/v1/generate", post(handlers::generate))
        .route("/v1/generate/stream", post(handlers::stream))
        .layer(DefaultBodyLimit::max(models::MAX_REQUEST_BYTES))
        .with_state(node)
}
