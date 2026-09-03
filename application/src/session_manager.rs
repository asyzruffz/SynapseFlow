use std::sync::Arc;

use synapseflow_domain::execution::{CheckpointRef, SafeTraceId};
use synapseflow_domain::{
    AdmissionDecision, AuthenticatedPrincipal, AuthorizationDecision, CancellationResult,
    DomainError, DomainResult, ErrorCode, IdempotencyKey, ModelReference, PublicSessionId,
    PublicSessionState,
};
use synapseflow_ports::{
    ActiveSessionControl, AdmissionAccounting, AdmissionRequest, AuditEvent, AuditSink,
    CreateSessionResult, DurableSession, ModelAccessPolicy, RequestFingerprint,
    SessionIdentifierIssuer, SessionStore,
};

/// Payload-free inputs required to create one application-owned generation session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStartRequest {
    pub principal: AuthenticatedPrincipal,
    pub model: ModelReference,
    pub reserved_output_tokens: u16,
    pub idempotency_key: Option<IdempotencyKey>,
    pub request_fingerprint: Option<RequestFingerprint>,
    pub trace_id: Option<SafeTraceId>,
}

/// Durable result of beginning a session or replaying an equivalent request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStartResult {
    pub session: DurableSession,
    pub replayed: bool,
}

/// Result data persisted before an externally observable terminal event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTerminal {
    pub state: PublicSessionState,
    pub token_count: usize,
    pub failure: Option<ErrorCode>,
    pub cancellation: Option<CancellationResult>,
}

impl SessionTerminal {
    pub fn completed(token_count: usize) -> Self {
        Self {
            state: PublicSessionState::Completed,
            token_count,
            failure: None,
            cancellation: None,
        }
    }

    pub fn cancelled(result: CancellationResult) -> Self {
        Self {
            state: PublicSessionState::Cancelled,
            token_count: 0,
            failure: None,
            cancellation: Some(result),
        }
    }

    pub fn failed(error: DomainError) -> Self {
        Self {
            state: PublicSessionState::Failed,
            token_count: 0,
            failure: Some(error.code()),
            cancellation: None,
        }
    }
}

/// Application control-plane authority shared by local and sharded execution.
pub struct GenerationSessionManager {
    identifiers: Arc<dyn SessionIdentifierIssuer>,
    policy: Arc<dyn ModelAccessPolicy>,
    admission: Arc<dyn AdmissionAccounting>,
    sessions: Arc<dyn SessionStore>,
    active: Arc<dyn ActiveSessionControl>,
    audit: Arc<dyn AuditSink>,
}

impl GenerationSessionManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identifiers: Arc<dyn SessionIdentifierIssuer>,
        policy: Arc<dyn ModelAccessPolicy>,
        admission: Arc<dyn AdmissionAccounting>,
        sessions: Arc<dyn SessionStore>,
        active: Arc<dyn ActiveSessionControl>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            identifiers,
            policy,
            admission,
            sessions,
            active,
            audit,
        }
    }

    /// Authorizes, admits, persists, and audits a session before exposing it.
    pub fn begin(&self, request: SessionStartRequest) -> DomainResult<SessionStartResult> {
        let authorization = self
            .policy
            .authorize_generation(&request.principal, &request.model)?;
        if authorization == AuthorizationDecision::Denied {
            return Err(DomainError::AuthorizationDenied);
        }

        if let Some(existing) = self.find_replay(&request)? {
            return Ok(existing);
        }

        let id = self.identifiers.issue()?;
        let admission_request = AdmissionRequest {
            principal: request.principal.clone(),
            session_id: id.clone(),
            reserved_output_tokens: request.reserved_output_tokens,
        };
        match self.admission.admit(&admission_request)? {
            AdmissionDecision::Admitted => {}
            AdmissionDecision::Rejected(_) => return Err(DomainError::AdmissionUnavailable),
        }

        let candidate = DurableSession {
            id: id.clone(),
            owner: request.principal,
            model: request.model,
            idempotency_key: request.idempotency_key,
            request_fingerprint: request.request_fingerprint,
            state: PublicSessionState::Accepted,
            checkpoints: Vec::new(),
            trace_id: request.trace_id,
        };
        let created = self.sessions.create(candidate)?;
        let session = match created {
            CreateSessionResult::Created(session) => session,
            CreateSessionResult::Replayed(session) => {
                self.admission.release(&id)?;
                return Ok(SessionStartResult {
                    session,
                    replayed: true,
                });
            }
            CreateSessionResult::Conflict => {
                self.admission.release(&id)?;
                return Err(DomainError::DuplicateWork);
            }
        };
        self.audit
            .record(AuditEvent::NodeSession(node_audit(&session, 0, None, None)))?;

        Ok(SessionStartResult {
            session,
            replayed: false,
        })
    }

    /// Persists running state before the runtime starts producing output.
    pub fn mark_running(&self, session_id: &PublicSessionId) -> DomainResult<DurableSession> {
        self.transition(session_id, PublicSessionState::Running)
    }

    /// Persists a checkpoint reference before it can be used for recovery.
    pub fn record_checkpoint(
        &self,
        session_id: &PublicSessionId,
        checkpoint: CheckpointRef,
    ) -> DomainResult<DurableSession> {
        let mut session = self.load(session_id)?;
        if session.state != PublicSessionState::Running {
            return Err(DomainError::SessionStateInvalid);
        }
        session.checkpoints.push(checkpoint);
        self.sessions.replace(session.clone())?;
        Ok(session)
    }

    /// Verifies ownership, persists cancellation intent, then signals active work.
    pub fn cancel(
        &self,
        actor: &AuthenticatedPrincipal,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult> {
        let session = self.load(session_id)?;
        if session.owner.pseudonym() != actor.pseudonym()
            && !actor
                .scopes()
                .contains(synapseflow_domain::GrantedScope::CancelAny)
        {
            return Err(DomainError::AuthorizationDenied);
        }
        if session.state.is_terminal() {
            return Ok(CancellationResult::AlreadyTerminal(session.state));
        }
        if session.state == PublicSessionState::Cancelling {
            return Ok(CancellationResult::AlreadyCancelling);
        }
        self.transition(session_id, PublicSessionState::Cancelling)?;
        self.active.request_cancellation(session_id)
    }

    pub(crate) fn activate(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<std::sync::Arc<dyn synapseflow_ports::ExecutionCancellation>> {
        self.active.activate(session_id)
    }

    pub(crate) fn deactivate(&self, session_id: &PublicSessionId) -> DomainResult<()> {
        self.active.deactivate(session_id)
    }

    pub(crate) fn cancelled_result(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult> {
        let session = self.load(session_id)?;
        match session.state {
            PublicSessionState::Cancelling => Ok(CancellationResult::Requested),
            state @ (PublicSessionState::Completed
            | PublicSessionState::Cancelled
            | PublicSessionState::Failed) => Ok(CancellationResult::AlreadyTerminal(state)),
            PublicSessionState::Accepted | PublicSessionState::Running => {
                Ok(CancellationResult::Requested)
            }
        }
    }

    /// Persists a terminal state, records its terminal audit, then releases admission.
    pub fn finish(
        &self,
        session_id: &PublicSessionId,
        terminal: SessionTerminal,
    ) -> DomainResult<DurableSession> {
        if !terminal.state.is_terminal() {
            return Err(DomainError::SessionStateInvalid);
        }
        let session = self.transition(session_id, terminal.state)?;
        self.audit.record(AuditEvent::NodeSession(node_audit(
            &session,
            terminal.token_count,
            terminal.failure,
            terminal.cancellation,
        )))?;
        self.admission.release(session_id)?;
        Ok(session)
    }

    fn find_replay(
        &self,
        request: &SessionStartRequest,
    ) -> DomainResult<Option<SessionStartResult>> {
        let Some(key) = request.idempotency_key.as_ref() else {
            return Ok(None);
        };
        let Some(session) = self.sessions.find_by_idempotency(&request.principal, key)? else {
            return Ok(None);
        };
        if session.request_fingerprint == request.request_fingerprint {
            Ok(Some(SessionStartResult {
                session,
                replayed: true,
            }))
        } else {
            Err(DomainError::DuplicateWork)
        }
    }

    fn transition(
        &self,
        session_id: &PublicSessionId,
        next: PublicSessionState,
    ) -> DomainResult<DurableSession> {
        let mut session = self.load(session_id)?;
        session.state = session.state.transition(next)?;
        self.sessions.replace(session.clone())?;
        Ok(session)
    }

    fn load(&self, session_id: &PublicSessionId) -> DomainResult<DurableSession> {
        self.sessions
            .load(session_id)?
            .ok_or(DomainError::SessionUnavailable)
    }
}

fn node_audit(
    session: &DurableSession,
    token_count: usize,
    failure: Option<ErrorCode>,
    cancellation: Option<CancellationResult>,
) -> synapseflow_ports::NodeSessionAudit {
    synapseflow_ports::NodeSessionAudit {
        principal: session.owner.clone(),
        authorization: AuthorizationDecision::Authorized,
        admission: AdmissionDecision::Admitted,
        session_id: session.id.clone(),
        trace_id: session.trace_id.clone(),
        model: session.model.clone(),
        token_count,
        failure,
        cancellation,
    }
}
