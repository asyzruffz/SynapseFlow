use std::sync::{Arc, Mutex};

use synapseflow_adapter_in_memory::InMemoryAuditSink;
use synapseflow_domain::execution::{CheckpointRef, FrameSequence, SessionId, StreamId};
use synapseflow_domain::{
    AdmissionDecision, AuthenticatedPrincipal, AuthorizationDecision, CancellationResult,
    DomainError, DomainResult, GrantedScope, GrantedScopes, IdempotencyKey, ModelReference,
    PrincipalPseudonym, PublicSessionId, PublicSessionState,
};
use synapseflow_ports::{
    ActiveSessionControl, AdmissionAccounting, AdmissionRequest, CreateSessionResult,
    DurableSession, ModelAccessPolicy, RequestFingerprint, SessionIdentifierIssuer, SessionStore,
};

use crate::{GenerationSessionManager, SessionStartRequest, SessionTerminal};

struct Identifiers(Mutex<u64>);

impl SessionIdentifierIssuer for Identifiers {
    fn issue(&self) -> DomainResult<PublicSessionId> {
        let mut next = self.0.lock().map_err(|_| DomainError::PersistenceFailure)?;
        *next += 1;
        PublicSessionId::new(format!("node-session-{next:016}"))
    }
}

struct Policy;

impl ModelAccessPolicy for Policy {
    fn authorize_generation(
        &self,
        _: &AuthenticatedPrincipal,
        _: &ModelReference,
    ) -> DomainResult<AuthorizationDecision> {
        Ok(AuthorizationDecision::Authorized)
    }
}

#[derive(Default)]
struct Admission {
    admitted: Mutex<Vec<PublicSessionId>>,
    released: Mutex<Vec<PublicSessionId>>,
}

impl AdmissionAccounting for Admission {
    fn admit(&self, request: &AdmissionRequest) -> DomainResult<AdmissionDecision> {
        self.admitted
            .lock()
            .map_err(|_| DomainError::AdmissionUnavailable)?
            .push(request.session_id.clone());
        Ok(AdmissionDecision::Admitted)
    }

    fn release(&self, session_id: &PublicSessionId) -> DomainResult<()> {
        self.released
            .lock()
            .map_err(|_| DomainError::AdmissionUnavailable)?
            .push(session_id.clone());
        Ok(())
    }
}

#[derive(Default)]
struct Store(Mutex<Vec<DurableSession>>);

impl SessionStore for Store {
    fn create(&self, session: DurableSession) -> DomainResult<CreateSessionResult> {
        let mut sessions = self.0.lock().map_err(|_| DomainError::PersistenceFailure)?;
        if sessions.iter().any(|existing| existing.id == session.id) {
            return Ok(CreateSessionResult::Conflict);
        }
        if let Some(key) = session.idempotency_key.as_ref() {
            if let Some(existing) = sessions.iter().find(|existing| {
                existing.owner.pseudonym() == session.owner.pseudonym()
                    && existing.idempotency_key.as_ref() == Some(key)
            }) {
                return if existing.request_fingerprint == session.request_fingerprint {
                    Ok(CreateSessionResult::Replayed(existing.clone()))
                } else {
                    Ok(CreateSessionResult::Conflict)
                };
            }
        }
        sessions.push(session.clone());
        Ok(CreateSessionResult::Created(session))
    }

    fn load(&self, session_id: &PublicSessionId) -> DomainResult<Option<DurableSession>> {
        Ok(self
            .0
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)?
            .iter()
            .find(|session| &session.id == session_id)
            .cloned())
    }

    fn find_by_idempotency(
        &self,
        owner: &AuthenticatedPrincipal,
        key: &IdempotencyKey,
    ) -> DomainResult<Option<DurableSession>> {
        Ok(self
            .0
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)?
            .iter()
            .find(|session| {
                session.owner.pseudonym() == owner.pseudonym()
                    && session.idempotency_key.as_ref() == Some(key)
            })
            .cloned())
    }

    fn replace(&self, session: DurableSession) -> DomainResult<()> {
        let mut sessions = self.0.lock().map_err(|_| DomainError::PersistenceFailure)?;
        let existing = sessions
            .iter_mut()
            .find(|existing| existing.id == session.id)
            .ok_or(DomainError::SessionUnavailable)?;
        *existing = session;
        Ok(())
    }
}

#[derive(Default)]
struct Active(Mutex<Vec<PublicSessionId>>);

impl ActiveSessionControl for Active {
    fn request_cancellation(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult> {
        self.0
            .lock()
            .map_err(|_| DomainError::SessionUnavailable)?
            .push(session_id.clone());
        Ok(CancellationResult::Requested)
    }
}

fn principal(name: &str, scopes: impl IntoIterator<Item = GrantedScope>) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(
        PrincipalPseudonym::new(name.to_owned()).expect("fixture pseudonym should be valid"),
        GrantedScopes::new(scopes),
    )
}

fn request(principal: AuthenticatedPrincipal) -> SessionStartRequest {
    SessionStartRequest {
        principal,
        model: ModelReference::parse(format!(
            "registry://fixtures/model@sha256:{}",
            "a".repeat(64)
        ))
        .expect("fixture model should be valid"),
        reserved_output_tokens: 4,
        idempotency_key: Some(
            IdempotencyKey::new("idempotency-00001".to_owned())
                .expect("fixture key should be valid"),
        ),
        request_fingerprint: Some(RequestFingerprint::new([7; 32])),
        trace_id: None,
    }
}

fn manager(
    store: Arc<Store>,
    admission: Arc<Admission>,
    active: Arc<Active>,
    audit: Arc<InMemoryAuditSink>,
) -> GenerationSessionManager {
    GenerationSessionManager::new(
        Arc::new(Identifiers(Mutex::new(0))),
        Arc::new(Policy),
        admission,
        store,
        active,
        audit,
    )
}

#[test]
fn persists_state_and_checkpoint_before_terminal_audit_and_capacity_release() {
    let store = Arc::new(Store::default());
    let admission = Arc::new(Admission::default());
    let active = Arc::new(Active::default());
    let audit = Arc::new(InMemoryAuditSink::default());
    let manager = manager(store.clone(), admission.clone(), active, audit.clone());
    let created = manager
        .begin(request(principal("owner_0001", [GrantedScope::Generate])))
        .expect("session should begin");

    let running = manager
        .mark_running(&created.session.id)
        .expect("running state should persist");
    assert_eq!(running.state, PublicSessionState::Running);
    let checkpoint = CheckpointRef {
        session_id: SessionId::new("execution-session-0001".to_owned())
            .expect("fixture execution session should be valid"),
        stream_id: StreamId::new(1).expect("fixture stream should be valid"),
        sequence: FrameSequence::initial(),
    };
    assert_eq!(
        manager
            .record_checkpoint(&created.session.id, checkpoint.clone())
            .expect("checkpoint should persist")
            .checkpoints,
        vec![checkpoint]
    );
    let completed = manager
        .finish(&created.session.id, SessionTerminal::completed(3))
        .expect("terminal state should persist and audit");

    assert_eq!(completed.state, PublicSessionState::Completed);
    assert_eq!(
        admission.released.lock().expect("release lock").as_slice(),
        [created.session.id.clone()]
    );
    assert_eq!(audit.events().expect("audit events").len(), 2);
    assert_eq!(
        store
            .load(&created.session.id)
            .expect("store should load")
            .expect("session should remain observable")
            .state,
        PublicSessionState::Completed
    );
}

#[test]
fn replays_equivalent_requests_and_enforces_owner_only_cancellation() {
    let store = Arc::new(Store::default());
    let admission = Arc::new(Admission::default());
    let active = Arc::new(Active::default());
    let audit = Arc::new(InMemoryAuditSink::default());
    let manager = manager(store, admission.clone(), active.clone(), audit);
    let owner = principal("owner_0001", [GrantedScope::Generate]);
    let created = manager
        .begin(request(owner.clone()))
        .expect("session should begin");
    let replayed = manager
        .begin(request(owner.clone()))
        .expect("equivalent request should replay");

    assert!(replayed.replayed);
    assert_eq!(replayed.session.id, created.session.id);
    assert_eq!(admission.admitted.lock().expect("admission lock").len(), 1);
    assert!(matches!(
        manager.cancel(
            &principal("other_0001", [GrantedScope::Generate]),
            &created.session.id
        ),
        Err(DomainError::AuthorizationDenied)
    ));
    assert_eq!(
        manager
            .cancel(&owner, &created.session.id)
            .expect("owner should cancel"),
        CancellationResult::Requested
    );
    assert_eq!(
        active.0.lock().expect("active lock").as_slice(),
        [created.session.id.clone()]
    );
}
