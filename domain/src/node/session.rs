use crate::{DomainError, DomainResult};
use std::fmt;

/// Opaque client-visible identifier for an application-owned generation session.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PublicSessionId(String);

impl PublicSessionId {
    pub fn new(value: String) -> DomainResult<Self> {
        if is_safe_identifier(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::PublicSessionInvalid)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PublicSessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Presentation-safe lifecycle state for one generation session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicSessionState {
    Accepted,
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
}

impl PublicSessionState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }

    pub fn transition(self, next: Self) -> DomainResult<Self> {
        let valid = matches!(
            (self, next),
            (
                Self::Accepted,
                Self::Running | Self::Cancelling | Self::Failed
            ) | (
                Self::Running,
                Self::Cancelling | Self::Completed | Self::Cancelled | Self::Failed
            ) | (Self::Cancelling, Self::Cancelled | Self::Failed)
        );
        if valid {
            Ok(next)
        } else {
            Err(DomainError::SessionStateInvalid)
        }
    }
}

impl fmt::Display for PublicSessionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            PublicSessionState::Accepted => "accepted",
            PublicSessionState::Running => "running",
            PublicSessionState::Cancelling => "cancelling",
            PublicSessionState::Completed => "completed",
            PublicSessionState::Cancelled => "cancelled",
            PublicSessionState::Failed => "failed",
        };
        formatter.write_str(state)
    }
}

/// Safe outcome of an idempotent cancellation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationResult {
    Requested,
    AlreadyCancelling,
    AlreadyTerminal(PublicSessionState),
}

/// Bounded opaque request key used to deduplicate one principal's submission.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.len() >= 16 && is_safe_identifier(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::IdempotencyKeyInvalid)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::{CancellationResult, IdempotencyKey, PublicSessionId, PublicSessionState};
    use crate::DomainError;

    #[test]
    fn accepts_an_opaque_public_session_identifier() {
        let session = PublicSessionId::new("session-0000006F4D".to_owned())
            .expect("fixture session identifier should be valid");

        assert_eq!(session.as_str(), "session-0000006F4D");
    }

    #[test]
    fn rejects_non_opaque_session_identifiers() {
        assert_eq!(
            PublicSessionId::new("session/6F4D".to_owned()),
            Err(DomainError::PublicSessionInvalid)
        );
    }

    #[test]
    fn distinguishes_terminal_cancellation_from_a_new_request() {
        assert_eq!(
            CancellationResult::AlreadyTerminal(PublicSessionState::Completed),
            CancellationResult::AlreadyTerminal(PublicSessionState::Completed)
        );
        assert!(PublicSessionState::Cancelled.is_terminal());
        assert!(!PublicSessionState::Cancelling.is_terminal());
    }

    #[test]
    fn bounds_idempotency_keys() {
        assert_eq!(
            IdempotencyKey::new("too-short".to_owned()),
            Err(DomainError::IdempotencyKeyInvalid)
        );
        assert!(IdempotencyKey::new("idempotency-00001".to_owned()).is_ok());
    }
}
