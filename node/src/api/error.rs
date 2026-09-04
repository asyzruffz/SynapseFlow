use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use synapseflow_domain::DomainError;

use super::session_response::ErrorRepresentation;

pub struct ApiError {
    status: StatusCode,
    error: DomainError,
}

impl ApiError {
    pub fn from_domain(error: DomainError) -> Self {
        let status = match error {
            DomainError::AuthenticationInvalid | DomainError::ScopeInvalid => {
                StatusCode::UNAUTHORIZED
            }
            DomainError::AuthorizationDenied => StatusCode::FORBIDDEN,
            DomainError::AdmissionUnavailable => StatusCode::TOO_MANY_REQUESTS,
            DomainError::DuplicateWork | DomainError::GenerationStreamInvalid => {
                StatusCode::CONFLICT
            }
            DomainError::SessionUnavailable => StatusCode::NOT_FOUND,
            DomainError::GenerationPolicyInvalid
            | DomainError::InvalidReference
            | DomainError::IdempotencyKeyInvalid => StatusCode::BAD_REQUEST,
            DomainError::PersistenceFailure | DomainError::AuditUnavailable => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self { status, error }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = match self.status {
            StatusCode::UNAUTHORIZED => "authentication failed",
            StatusCode::FORBIDDEN => "operation is not authorized",
            StatusCode::TOO_MANY_REQUESTS => "node admission is unavailable",
            StatusCode::CONFLICT => "request conflicts with an existing session",
            StatusCode::NOT_FOUND => "session is unavailable",
            StatusCode::BAD_REQUEST => "request is invalid",
            StatusCode::SERVICE_UNAVAILABLE => "service state is unavailable",
            _ => "request could not be completed",
        };
        (
            self.status,
            Json(ErrorRepresentation {
                code: self.error.code().to_string(),
                message,
            }),
        )
            .into_response()
    }
}
