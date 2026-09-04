use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{sse::KeepAlive, sse::Sse, IntoResponse, Response},
    Json,
};
use synapseflow_domain::{DomainError, PublicSessionId};

use super::{
    authentication::authenticate,
    event_stream::SessionEventStream,
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
        state
            .workflows
            .insert(
                started.session.id.clone(),
                synapseflow_kernel::Core::<synapseflow_kernel::SynapseFlow>::new(),
            )
            .and_then(|_| state.workflows.begin(&started.session.id))
            .map_err(ApiError::from_domain)?;
        execution_dispatch::start(
            state.dependencies.execution.clone(),
            started.clone(),
            generation,
            state.workflows.clone(),
        );
    }
    accepted(started)
}

/// Subscribes the session owner to bounded, live-only generation events.
pub(super) async fn session_events(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
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
    if session.state.is_terminal() {
        return Err(ApiError::from_domain(DomainError::GenerationStreamInvalid));
    }
    let receiver = state
        .workflows
        .subscribe_events(&session_id)
        .map_err(ApiError::from_domain)?;
    Ok(Sse::new(SessionEventStream::new(session_id, receiver)).keep_alive(KeepAlive::default()))
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
