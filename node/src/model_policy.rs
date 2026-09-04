use std::collections::BTreeSet;

use synapseflow_domain::{
    AuthenticatedPrincipal, AuthorizationDecision, DomainResult, GrantedScope, ModelReference,
};
use synapseflow_ports::ModelAccessPolicy;

/// Configuration-backed model policy for immutable manifest references only.
pub struct ConfiguredModelAccessPolicy {
    allowed_models: BTreeSet<ModelReference>,
}

impl ConfiguredModelAccessPolicy {
    pub fn new(allowed_models: BTreeSet<ModelReference>) -> Self {
        Self { allowed_models }
    }
}

impl ModelAccessPolicy for ConfiguredModelAccessPolicy {
    fn authorize_generation(
        &self,
        principal: &AuthenticatedPrincipal,
        model: &ModelReference,
    ) -> DomainResult<AuthorizationDecision> {
        Ok(
            if principal.scopes().contains(GrantedScope::Generate)
                && self.allowed_models.contains(model)
            {
                AuthorizationDecision::Authorized
            } else {
                AuthorizationDecision::Denied
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use synapseflow_domain::{
        AuthenticatedPrincipal, AuthorizationDecision, GrantedScope, GrantedScopes, ModelReference,
        PrincipalPseudonym,
    };
    use synapseflow_ports::ModelAccessPolicy;

    use super::ConfiguredModelAccessPolicy;

    #[test]
    fn requires_both_generation_scope_and_a_configured_immutable_model() {
        let model = ModelReference::parse(format!(
            "registry://fixtures/policy@sha256:{}",
            "a".repeat(64)
        ))
        .expect("fixture model should parse");
        let policy = ConfiguredModelAccessPolicy::new(BTreeSet::from([model.clone()]));
        let principal = AuthenticatedPrincipal::new(
            PrincipalPseudonym::new("principal-policy".to_owned()).expect("fixture principal"),
            GrantedScopes::new([GrantedScope::Generate]),
        );
        assert_eq!(
            policy.authorize_generation(&principal, &model),
            Ok(AuthorizationDecision::Authorized)
        );
    }
}
