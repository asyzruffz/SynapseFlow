use synapseflow_domain::{
    AdmissionDecision, AuthenticatedPrincipal, DomainResult, PublicSessionId,
};

/// Bounded work reservation requested after authentication and authorization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRequest {
    pub principal: AuthenticatedPrincipal,
    pub session_id: PublicSessionId,
    pub reserved_output_tokens: u16,
}

/// Atomically accounts for bounded node and per-principal capacity.
pub trait AdmissionAccounting: Send + Sync {
    fn admit(&self, request: &AdmissionRequest) -> DomainResult<AdmissionDecision>;

    fn release(&self, session_id: &PublicSessionId) -> DomainResult<()>;
}
