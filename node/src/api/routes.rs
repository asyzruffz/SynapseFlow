use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};

use crate::NodeDependencies;

use super::{
    sessions::{create_session, session_status},
    state::ApiState,
};

/// Installs the public v1 routes only when the CLI supplied a complete composition.
pub(crate) fn router(dependencies: Arc<NodeDependencies>, max_request_bytes: usize) -> Router {
    Router::new()
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/{session_id}", get(session_status))
        .layer(DefaultBodyLimit::max(max_request_bytes))
        .with_state(ApiState { dependencies })
}
