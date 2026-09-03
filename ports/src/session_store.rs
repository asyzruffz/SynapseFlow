use synapseflow_domain::execution::CheckpointRef;
use synapseflow_domain::execution::SafeTraceId;
use synapseflow_domain::{
    AuthenticatedPrincipal, CancellationResult, DomainResult, IdempotencyKey, ModelReference,
    PublicSessionId, PublicSessionState,
};

/// Fixed-length hash of a canonical create-session request, never its payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestFingerprint([u8; 32]);

impl RequestFingerprint {
    pub const fn new(value: [u8; 32]) -> Self {
        Self(value)
    }
}

/// Durable, presentation-safe state owned by the application session manager.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSession {
    pub id: PublicSessionId,
    pub owner: AuthenticatedPrincipal,
    pub model: ModelReference,
    pub idempotency_key: Option<IdempotencyKey>,
    pub request_fingerprint: Option<RequestFingerprint>,
    pub state: PublicSessionState,
    pub checkpoints: Vec<CheckpointRef>,
    pub trace_id: Option<SafeTraceId>,
}

/// Outcome of atomic durable session creation and idempotency reservation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateSessionResult {
    Created(DurableSession),
    Replayed(DurableSession),
    Conflict,
}

/// Persists application session state and checkpoint references before exposure.
pub trait SessionStore: Send + Sync {
    fn create(&self, session: DurableSession) -> DomainResult<CreateSessionResult>;

    fn load(&self, session_id: &PublicSessionId) -> DomainResult<Option<DurableSession>>;

    fn find_by_idempotency(
        &self,
        owner: &AuthenticatedPrincipal,
        key: &IdempotencyKey,
    ) -> DomainResult<Option<DurableSession>>;

    fn replace(&self, session: DurableSession) -> DomainResult<()>;
}

/// Issues collision-resistant application session identities without choosing a transport format.
pub trait SessionIdentifierIssuer: Send + Sync {
    fn issue(&self) -> DomainResult<PublicSessionId>;
}

/// Looks up and signals active work without owning authorization or durable state.
pub trait ActiveSessionControl: Send + Sync {
    fn request_cancellation(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult>;
}
