use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use synapseflow_application::SessionStartResult;
use synapseflow_domain::{CancellationResult, DomainError, PublicSessionState};

use super::ApiError;

/// Presentation-safe session state exposed by create and status endpoints.
#[derive(Serialize)]
pub(super) struct SessionRepresentation {
    session_id: String,
    state: String,
    replayed: bool,
}

#[derive(Serialize)]
pub(super) struct ErrorRepresentation {
    pub(super) code: String,
    pub(super) message: &'static str,
}

#[derive(Serialize)]
pub(super) struct CancellationRepresentation {
    session_id: String,
    outcome: &'static str,
}

pub(super) fn status(
    session_id: impl ToString,
    state: PublicSessionState,
) -> Json<SessionRepresentation> {
    Json(SessionRepresentation {
        session_id: session_id.to_string(),
        state: state.to_string(),
        replayed: false,
    })
}

pub(super) fn accepted(started: SessionStartResult) -> Result<Response, ApiError> {
    let location = HeaderValue::from_str(&format!("/v1/sessions/{}", started.session.id))
        .map_err(|_| ApiError::from_domain(DomainError::PublicSessionInvalid))?;
    let mut response = (
        StatusCode::ACCEPTED,
        Json(SessionRepresentation {
            session_id: started.session.id.to_string(),
            state: started.session.state.to_string(),
            replayed: started.replayed,
        }),
    )
        .into_response();
    response.headers_mut().insert(header::LOCATION, location);
    Ok(response)
}

pub(super) fn cancellation(
    session_id: impl ToString,
    result: CancellationResult,
) -> (StatusCode, Json<CancellationRepresentation>) {
    let (status, outcome) = match result {
        CancellationResult::Requested => (StatusCode::ACCEPTED, "requested"),
        CancellationResult::AlreadyCancelling => (StatusCode::OK, "already_cancelling"),
        CancellationResult::AlreadyTerminal(PublicSessionState::Completed) => {
            (StatusCode::OK, "already_completed")
        }
        CancellationResult::AlreadyTerminal(PublicSessionState::Cancelled) => {
            (StatusCode::OK, "already_cancelled")
        }
        CancellationResult::AlreadyTerminal(PublicSessionState::Failed) => {
            (StatusCode::OK, "already_failed")
        }
        CancellationResult::AlreadyTerminal(
            PublicSessionState::Accepted
            | PublicSessionState::Running
            | PublicSessionState::Cancelling,
        ) => (StatusCode::OK, "already_terminal"),
    };
    (
        status,
        Json(CancellationRepresentation {
            session_id: session_id.to_string(),
            outcome,
        }),
    )
}
