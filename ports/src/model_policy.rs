use synapseflow_domain::{
    AuthenticatedPrincipal, AuthorizationDecision, DomainResult, ModelReference,
};

/// Applies application-owned caller scope and immutable-model policy.
pub trait ModelAccessPolicy: Send + Sync {
    fn authorize_generation(
        &self,
        principal: &AuthenticatedPrincipal,
        model: &ModelReference,
    ) -> DomainResult<AuthorizationDecision>;
}
