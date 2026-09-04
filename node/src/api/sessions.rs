use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use synapseflow_domain::{DomainError, PublicSessionId};

use super::{
    authentication::authenticate,
    execution_dispatch,
    session_request::{idempotency_key, into_requests, CreateSessionBody},
    session_response::{accepted, status, SessionRepresentation},
    state::ApiState,
    ApiError,
};

/// Authenticates, durably accepts, and dispatches a new generation session.
pub(super) async fn create_session(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(body): Json<CreateSessionBody>,
) -> Result<Response, ApiError> {
    let principal = authenticate(&state, &headers).await?;
    let idempotency_key = idempotency_key(&headers).map_err(ApiError::from_domain)?;
    let (generation, start) =
        into_requests(principal, body, idempotency_key).map_err(ApiError::from_domain)?;
    let started = state
        .dependencies
        .sessions
        .begin(start)
        .map_err(ApiError::from_domain)?;

    if !started.replayed {
        execution_dispatch::start(
            state.dependencies.execution.clone(),
            started.clone(),
            generation,
        );
    }
    accepted(started)
}

/// Returns only durable presentation-safe state to the session owner.
pub(super) async fn session_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<SessionRepresentation>, ApiError> {
    let principal = authenticate(&state, &headers).await?;
    let session_id = PublicSessionId::new(session_id).map_err(ApiError::from_domain)?;
    let session = state
        .dependencies
        .sessions
        .session(&session_id)
        .map_err(ApiError::from_domain)?;
    if session.owner.pseudonym() != principal.pseudonym() {
        return Err(ApiError::from_domain(DomainError::AuthorizationDenied));
    }
    Ok(status(session.id, session.state))
}
