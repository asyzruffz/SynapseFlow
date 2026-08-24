use crate::{DomainError, DomainResult};

/// Lifecycle state owned by the distributed session manager.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionState {
    Created,
    Planned,
    Running,
    Retrying,
    Completing,
    Completed,
    Cancelling,
    Cancelled,
    Failed,
}

impl SessionState {
    pub fn transition(self, next: Self) -> DomainResult<Self> {
        let valid = matches!(
            (self, next),
            (Self::Created, Self::Planned)
                | (
                    Self::Planned,
                    Self::Running | Self::Failed | Self::Cancelling
                )
                | (
                    Self::Running,
                    Self::Retrying | Self::Completing | Self::Failed | Self::Cancelling
                )
                | (
                    Self::Retrying,
                    Self::Running | Self::Failed | Self::Cancelling
                )
                | (
                    Self::Completing,
                    Self::Completed | Self::Failed | Self::Cancelling
                )
                | (Self::Cancelling, Self::Cancelled)
                | (Self::Cancelled, Self::Cancelled)
        );
        if !valid {
            return Err(DomainError::SessionStateInvalid);
        }
        Ok(next)
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }

    /// Starts cancellation once, or confirms that a completed cancellation remains terminal.
    pub fn cancel(self) -> DomainResult<Self> {
        match self {
            Self::Created | Self::Planned | Self::Running | Self::Retrying | Self::Completing => {
                Ok(Self::Cancelling)
            }
            Self::Cancelling | Self::Cancelled => Ok(self),
            Self::Completed | Self::Failed => Err(DomainError::SessionStateInvalid),
        }
    }
}

/// Bounded number of recovery attempts remaining for one session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryBudget {
    remaining: u8,
}

impl RetryBudget {
    pub const fn new(retries: u8) -> Self {
        Self { remaining: retries }
    }

    pub const fn remaining(&self) -> u8 {
        self.remaining
    }

    pub fn consume(&mut self) -> DomainResult<()> {
        let Some(remaining) = self.remaining.checked_sub(1) else {
            return Err(DomainError::RetryExhausted);
        };
        self.remaining = remaining;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{RetryBudget, SessionState};
    use crate::DomainError;

    #[test]
    fn permits_only_documented_session_transitions() {
        let running = SessionState::Created
            .transition(SessionState::Planned)
            .and_then(|state| state.transition(SessionState::Running))
            .expect("created session should become running through planning");

        assert!(matches!(
            running.transition(SessionState::Completed),
            Err(DomainError::SessionStateInvalid)
        ));
        assert!(matches!(
            SessionState::Cancelled.transition(SessionState::Cancelled),
            Ok(SessionState::Cancelled)
        ));
    }

    #[test]
    fn bounds_retry_attempts() {
        let mut budget = RetryBudget::new(1);

        assert!(budget.consume().is_ok());
        assert_eq!(budget.remaining(), 0);
        assert!(matches!(budget.consume(), Err(DomainError::RetryExhausted)));
    }

    #[test]
    fn cancellation_is_idempotent_before_a_terminal_result() {
        assert!(matches!(
            SessionState::Running.cancel(),
            Ok(SessionState::Cancelling)
        ));
        assert!(matches!(
            SessionState::Cancelling.cancel(),
            Ok(SessionState::Cancelling)
        ));
    }
}
