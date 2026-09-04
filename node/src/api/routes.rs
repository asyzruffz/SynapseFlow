use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
    Router,
};

use crate::{AdmissionSettings, NodeDependencies, NodeWorkflowRegistry};

use super::{
    request_limits::SessionRequestLimits,
    sessions::{cancel_session, create_session, session_events, session_status},
    state::ApiState,
};

/// Installs the public v1 routes only when the CLI supplied a complete composition.
pub(crate) fn router(
    dependencies: Arc<NodeDependencies>,
    admission: &AdmissionSettings,
    workflows: Arc<NodeWorkflowRegistry>,
) -> Router {
    Router::new()
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/{session_id}", get(session_status))
        .route("/v1/sessions/{session_id}", delete(cancel_session))
        .route("/v1/sessions/{session_id}/events", get(session_events))
        .layer(DefaultBodyLimit::max(admission.max_request_bytes))
        .with_state(ApiState {
            dependencies,
            workflows,
            request_limits: SessionRequestLimits::from(admission),
        })
}
