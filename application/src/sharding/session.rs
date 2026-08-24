use std::sync::{Arc, Mutex};

use synapseflow_domain::execution::{
    CheckpointRef, RemainingDeadline, RetryBudget, SessionId, SessionState,
};
use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::{AuditEvent, AuditSink, ShardSessionOutcome};

use super::ExecutionRoute;

/// Bounded, opaque key used to deduplicate work submitted by a caller.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: String) -> DomainResult<Self> {
        let valid = (16..=128).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
        if !valid {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Immutable configuration owned by one session-manager entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionConfiguration {
    pub idempotency_key: IdempotencyKey,
    pub session_id: SessionId,
    pub route: ExecutionRoute,
    pub remaining_deadline: RemainingDeadline,
    pub retry_budget: RetryBudget,
}

/// Safe snapshot of one session's state and session-owned checkpoint selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSnapshot {
    pub idempotency_key: IdempotencyKey,
    pub session_id: SessionId,
    pub route: ExecutionRoute,
    pub state: SessionState,
    pub remaining_deadline: RemainingDeadline,
    pub retries_remaining: u8,
    pub retry_count: u8,
    pub fallback_count: u8,
    pub checkpoint: Option<CheckpointRef>,
}

/// Recovery instruction selected exclusively from session-owned checkpoint state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryAttempt {
    pub session: SessionSnapshot,
    pub checkpoint: CheckpointRef,
}

struct ManagedSession {
    configuration: SessionConfiguration,
    state: SessionState,
    checkpoint: Option<CheckpointRef>,
    retry_count: u8,
    fallback_count: u8,
    audit_emitted: bool,
}

impl ManagedSession {
    fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            idempotency_key: self.configuration.idempotency_key.clone(),
            session_id: self.configuration.session_id.clone(),
            route: self.configuration.route.clone(),
            state: self.state,
            remaining_deadline: self.configuration.remaining_deadline,
            retries_remaining: self.configuration.retry_budget.remaining(),
            retry_count: self.retry_count,
            fallback_count: self.fallback_count,
            checkpoint: self.checkpoint.clone(),
        }
    }
}

/// The sole owner of sharded-session state, retries, and checkpoint selection.
pub struct SessionManager {
    audit: Arc<dyn AuditSink>,
    sessions: Mutex<Vec<ManagedSession>>,
}

impl SessionManager {
    pub fn new(audit: Arc<dyn AuditSink>) -> Self {
        Self {
            audit,
            sessions: Mutex::new(Vec::new()),
        }
    }

    /// Creates one session or returns the existing same-session request idempotently.
    pub fn begin(&self, configuration: SessionConfiguration) -> DomainResult<SessionSnapshot> {
        if !configuration.route.strategy.is_layer_range()
            || configuration.route.assignments.len() != 2
        {
            return Err(DomainError::ShardPlanInvalid);
        }
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        if let Some(existing) = sessions
            .iter()
            .find(|session| session.configuration.idempotency_key == configuration.idempotency_key)
        {
            return if existing.configuration.session_id == configuration.session_id {
                Ok(existing.snapshot())
            } else {
                Err(DomainError::DuplicateWork)
            };
        }
        if sessions
            .iter()
            .any(|session| session.configuration.session_id == configuration.session_id)
        {
            return Err(DomainError::DuplicateWork);
        }
        let session = ManagedSession {
            configuration,
            state: SessionState::Created,
            checkpoint: None,
            retry_count: 0,
            fallback_count: 0,
            audit_emitted: false,
        };
        let snapshot = session.snapshot();
        sessions.push(session);
        Ok(snapshot)
    }

    pub fn mark_planned(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        self.transition(key, SessionState::Planned)
    }

    pub fn start(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        self.transition(key, SessionState::Running)
    }

    /// Records only a bounded checkpoint reference; raw activations remain outside this API.
    pub fn record_checkpoint(
        &self,
        key: &IdempotencyKey,
        checkpoint: CheckpointRef,
    ) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        if session.state != SessionState::Running
            || checkpoint.session_id != session.configuration.session_id
        {
            return Err(DomainError::SessionStateInvalid);
        }
        if session.checkpoint.as_ref().is_some_and(|current| {
            current.stream_id == checkpoint.stream_id
                && current.sequence.value() >= checkpoint.sequence.value()
        }) {
            return Err(DomainError::FrameSequenceInvalid);
        }
        session.checkpoint = Some(checkpoint);
        Ok(session.snapshot())
    }

    /// Consumes one retry and reuses the internally selected latest checkpoint.
    pub fn retry_from_latest_checkpoint(
        &self,
        key: &IdempotencyKey,
        used_fallback: bool,
    ) -> DomainResult<RecoveryAttempt> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        if matches!(
            session.state,
            SessionState::Cancelling | SessionState::Cancelled
        ) {
            return Err(DomainError::SessionCancelled);
        }
        if session.state != SessionState::Running {
            return Err(DomainError::SessionStateInvalid);
        }
        let checkpoint = session
            .checkpoint
            .clone()
            .ok_or(DomainError::ReplicaRecoveryFailed)?;
        session.configuration.retry_budget.consume()?;
        session.state = session.state.transition(SessionState::Retrying)?;
        session.retry_count = session.retry_count.saturating_add(1);
        if used_fallback {
            session.fallback_count = session.fallback_count.saturating_add(1);
        }
        session.state = session.state.transition(SessionState::Running)?;
        Ok(RecoveryAttempt {
            session: session.snapshot(),
            checkpoint,
        })
    }

    /// Starts idempotent cancellation; callers complete cleanup with `finish_cancellation`.
    pub fn cancel(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        session.state = session.state.cancel()?;
        Ok(session.snapshot())
    }

    pub fn finish_cancellation(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        self.finish(key, SessionState::Cancelled)
    }

    pub fn complete(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        session.state = session.state.transition(SessionState::Completing)?;
        session.state = session.state.transition(SessionState::Completed)?;
        self.audit_terminal(session)?;
        Ok(session.snapshot())
    }

    pub fn fail(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        self.finish(key, SessionState::Failed)
    }

    /// Marks the active request failed because no remaining deadline exists.
    pub fn expire(&self, key: &IdempotencyKey) -> DomainResult<()> {
        self.fail(key)?;
        Err(DomainError::DeadlineExceeded)
    }

    pub fn snapshot(&self, key: &IdempotencyKey) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        Ok(find_session(&mut sessions, key)?.snapshot())
    }

    fn transition(
        &self,
        key: &IdempotencyKey,
        next: SessionState,
    ) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        session.state = session.state.transition(next)?;
        Ok(session.snapshot())
    }

    fn finish(
        &self,
        key: &IdempotencyKey,
        terminal: SessionState,
    ) -> DomainResult<SessionSnapshot> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        let session = find_session(&mut sessions, key)?;
        session.state = session.state.transition(terminal)?;
        self.audit_terminal(session)?;
        Ok(session.snapshot())
    }

    fn audit_terminal(&self, session: &mut ManagedSession) -> DomainResult<()> {
        if session.audit_emitted {
            return Ok(());
        }
        let outcome = match session.state {
            SessionState::Completed if session.fallback_count > 0 => ShardSessionOutcome::Recovered,
            SessionState::Completed => ShardSessionOutcome::Completed,
            SessionState::Cancelled => ShardSessionOutcome::Cancelled,
            SessionState::Failed => ShardSessionOutcome::Failed,
            _ => return Err(DomainError::SessionStateInvalid),
        };
        for assignment in &session.configuration.route.assignments {
            self.audit.record(AuditEvent::ShardSessionFinished {
                model: assignment.target.model.clone(),
                shard: assignment.target.shard.clone(),
                worker: assignment.primary.clone(),
                session_id: session.configuration.session_id.clone(),
                outcome,
                retry_count: session.retry_count,
                fallback_count: session.fallback_count,
            })?;
        }
        session.audit_emitted = true;
        Ok(())
    }
}

fn find_session<'a>(
    sessions: &'a mut [ManagedSession],
    key: &IdempotencyKey,
) -> DomainResult<&'a mut ManagedSession> {
    sessions
        .iter_mut()
        .find(|session| session.configuration.idempotency_key == *key)
        .ok_or(DomainError::SessionStateInvalid)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use synapseflow_adapter_in_memory::InMemoryAuditSink;
    use synapseflow_domain::execution::{
        FrameSequence, RetryBudget, SessionId, SessionState, StreamId,
    };
    use synapseflow_domain::DomainError;
    use synapseflow_ports::{AuditEvent, ShardSessionOutcome};

    use super::{IdempotencyKey, SessionConfiguration, SessionManager};
    use crate::sharding::{test_support, ShardPlanner};

    fn configuration(retries: u8) -> SessionConfiguration {
        let manifest = test_support::manifest(1);
        let route = ShardPlanner::new(test_support::directory(&manifest))
            .plan(&manifest)
            .expect("fixture route should plan");
        SessionConfiguration {
            idempotency_key: IdempotencyKey::new("idempotency-00001".to_owned())
                .expect("fixture idempotency key is valid"),
            session_id: SessionId::new("session-00000001".to_owned())
                .expect("fixture session id is valid"),
            route,
            remaining_deadline: synapseflow_domain::execution::RemainingDeadline::new(
                Duration::from_millis(500),
            )
            .expect("fixture deadline is valid"),
            retry_budget: RetryBudget::new(retries),
        }
    }

    fn start(manager: &SessionManager, configuration: &SessionConfiguration) {
        manager
            .begin(configuration.clone())
            .expect("session should begin");
        manager
            .mark_planned(&configuration.idempotency_key)
            .expect("session should plan");
        manager
            .start(&configuration.idempotency_key)
            .expect("session should start");
    }

    #[test]
    fn owns_lifecycle_idempotency_and_payload_free_terminal_audits() {
        let audit = Arc::new(InMemoryAuditSink::default());
        let manager = SessionManager::new(audit.clone());
        let configuration = configuration(1);

        start(&manager, &configuration);
        assert_eq!(
            manager
                .snapshot(&configuration.idempotency_key)
                .expect("snapshot should exist")
                .state,
            SessionState::Running
        );
        let repeated = manager
            .begin(configuration.clone())
            .expect("same key and session must be idempotent");
        assert_eq!(repeated.state, SessionState::Running);

        let completed = manager
            .complete(&configuration.idempotency_key)
            .expect("running session should complete");
        assert_eq!(completed.state, SessionState::Completed);
        let events = audit.events().expect("audit events should be readable");
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| matches!(
            event,
            AuditEvent::ShardSessionFinished {
                outcome: ShardSessionOutcome::Completed,
                retry_count: 0,
                fallback_count: 0,
                ..
            }
        )));

        let mut conflicting = configuration.clone();
        conflicting.session_id = SessionId::new("session-00000002".to_owned())
            .expect("second fixture session id is valid");
        assert!(matches!(
            manager.begin(conflicting),
            Err(DomainError::DuplicateWork)
        ));

        let mut duplicate_session = configuration;
        duplicate_session.idempotency_key = IdempotencyKey::new("idempotency-00002".to_owned())
            .expect("second fixture idempotency key is valid");
        assert!(matches!(
            manager.begin(duplicate_session),
            Err(DomainError::DuplicateWork)
        ));
    }

    #[test]
    fn propagates_deadlines_and_bounds_checkpoint_retry_recovery() {
        let manager = SessionManager::new(Arc::new(InMemoryAuditSink::default()));
        let configuration = configuration(1);
        start(&manager, &configuration);
        let checkpoint = synapseflow_domain::execution::CheckpointRef {
            session_id: configuration.session_id.clone(),
            stream_id: StreamId::new(1).expect("fixture stream is valid"),
            sequence: FrameSequence::initial(),
        };
        manager
            .record_checkpoint(&configuration.idempotency_key, checkpoint.clone())
            .expect("running session should own the checkpoint");
        let recovery = manager
            .retry_from_latest_checkpoint(&configuration.idempotency_key, true)
            .expect("one retry should succeed");
        assert_eq!(recovery.checkpoint, checkpoint);
        assert_eq!(recovery.session.retry_count, 1);
        assert_eq!(recovery.session.fallback_count, 1);
        assert_eq!(recovery.session.retries_remaining, 0);
        assert_eq!(
            recovery.session.remaining_deadline,
            configuration.remaining_deadline
        );
        assert!(matches!(
            manager.retry_from_latest_checkpoint(&configuration.idempotency_key, true),
            Err(DomainError::RetryExhausted)
        ));
    }

    #[test]
    fn handles_repeated_cancellation_and_expired_deadlines_safely() {
        let manager = SessionManager::new(Arc::new(InMemoryAuditSink::default()));
        let cancellation_configuration = configuration(0);
        start(&manager, &cancellation_configuration);
        assert_eq!(
            manager
                .cancel(&cancellation_configuration.idempotency_key)
                .expect("cancellation should start")
                .state,
            SessionState::Cancelling
        );
        assert_eq!(
            manager
                .cancel(&cancellation_configuration.idempotency_key)
                .expect("cancellation should be idempotent")
                .state,
            SessionState::Cancelling
        );
        assert_eq!(
            manager
                .finish_cancellation(&cancellation_configuration.idempotency_key)
                .expect("cancellation should finish")
                .state,
            SessionState::Cancelled
        );
        assert!(matches!(
            manager
                .retry_from_latest_checkpoint(&cancellation_configuration.idempotency_key, false),
            Err(DomainError::SessionCancelled)
        ));

        let expired = SessionManager::new(Arc::new(InMemoryAuditSink::default()));
        let expired_configuration = configuration(0);
        expired
            .begin(expired_configuration.clone())
            .expect("session should begin");
        expired
            .mark_planned(&expired_configuration.idempotency_key)
            .expect("session should plan");
        assert!(matches!(
            expired.expire(&expired_configuration.idempotency_key),
            Err(DomainError::DeadlineExceeded)
        ));
        assert_eq!(
            expired
                .snapshot(&expired_configuration.idempotency_key)
                .expect("expired session should remain observable")
                .state,
            SessionState::Failed
        );
    }
}
