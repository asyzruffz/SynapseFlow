use std::time::{Duration, Instant};

use crate::{DomainError, DomainResult, GenerationPolicy, ModelReference};

/// A request that the application service can execute without knowing its transport.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerationRequest {
    pub model: ModelReference,
    pub prompt: String,
    pub policy: GenerationPolicy,
    deadline: Option<Instant>,
}

impl GenerationRequest {
    pub fn new(
        model: ModelReference,
        prompt: String,
        policy: GenerationPolicy,
    ) -> DomainResult<Self> {
        if prompt.is_empty() {
            return Err(DomainError::GenerationPolicyInvalid);
        }

        Ok(Self {
            model,
            prompt,
            policy,
            deadline: None,
        })
    }

    /// Applies a bounded, monotonic deadline to this request.
    pub fn with_deadline_after(mut self, duration: Duration) -> DomainResult<Self> {
        if duration.is_zero() {
            return Err(DomainError::GenerationPolicyInvalid);
        }
        self.deadline = Some(
            Instant::now()
                .checked_add(duration)
                .ok_or(DomainError::GenerationPolicyInvalid)?,
        );
        Ok(self)
    }

    /// Returns whether the caller-provided deadline has elapsed.
    pub fn deadline_expired(&self) -> bool {
        self.deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    /// Returns the caller's remaining deadline budget, when one was supplied.
    pub fn remaining_deadline(&self) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }
}
