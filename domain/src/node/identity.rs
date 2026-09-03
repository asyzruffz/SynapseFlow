use std::collections::BTreeSet;

use crate::{DomainError, DomainResult};

/// Stable, privacy-safe identifier for a verified caller.
///
/// Identity adapters derive this from the issuer subject without retaining the
/// bearer token or raw identity claim in the application control plane.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PrincipalPseudonym(String);

impl PrincipalPseudonym {
    pub fn new(value: String) -> DomainResult<Self> {
        if is_safe_identifier(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::PrincipalInvalid)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Node capabilities granted to a verified principal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GrantedScope {
    Generate,
    CancelAny,
    Observe,
}

/// Safe result of applying a generation or session-ownership policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Authorized,
    Denied,
}

impl GrantedScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generate => "synapseflow:generate",
            Self::CancelAny => "synapseflow:cancel:any",
            Self::Observe => "synapseflow:observe",
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        match value {
            "synapseflow:generate" => Ok(Self::Generate),
            "synapseflow:cancel:any" => Ok(Self::CancelAny),
            "synapseflow:observe" => Ok(Self::Observe),
            _ => Err(DomainError::ScopeInvalid),
        }
    }
}

/// Deduplicated node capabilities from a verified access token.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GrantedScopes(BTreeSet<GrantedScope>);

impl GrantedScopes {
    pub fn new(scopes: impl IntoIterator<Item = GrantedScope>) -> Self {
        Self(scopes.into_iter().collect())
    }

    pub fn contains(&self, scope: GrantedScope) -> bool {
        self.0.contains(&scope)
    }

    pub fn iter(&self) -> impl Iterator<Item = GrantedScope> + '_ {
        self.0.iter().copied()
    }
}

/// Framework-independent result of successful identity verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedPrincipal {
    pseudonym: PrincipalPseudonym,
    scopes: GrantedScopes,
}

impl AuthenticatedPrincipal {
    pub const fn new(pseudonym: PrincipalPseudonym, scopes: GrantedScopes) -> Self {
        Self { pseudonym, scopes }
    }

    pub const fn pseudonym(&self) -> &PrincipalPseudonym {
        &self.pseudonym
    }

    pub const fn scopes(&self) -> &GrantedScopes {
        &self.scopes
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
    use super::{AuthenticatedPrincipal, GrantedScope, GrantedScopes, PrincipalPseudonym};
    use crate::DomainError;

    #[test]
    fn accepts_a_safe_pseudonym_and_deduplicates_scopes() {
        let pseudonym = PrincipalPseudonym::new("principal_6F4D".to_owned())
            .expect("fixture pseudonym should be valid");
        let scopes = GrantedScopes::new([GrantedScope::Generate, GrantedScope::Generate]);
        let principal = AuthenticatedPrincipal::new(pseudonym, scopes);

        assert_eq!(principal.pseudonym().as_str(), "principal_6F4D");
        assert!(principal.scopes().contains(GrantedScope::Generate));
        assert_eq!(principal.scopes().iter().count(), 1);
    }

    #[test]
    fn rejects_raw_or_unbounded_principal_values_and_unknown_scopes() {
        assert_eq!(
            PrincipalPseudonym::new("user@example.com".to_owned()),
            Err(DomainError::PrincipalInvalid)
        );
        assert_eq!(GrantedScope::parse("admin"), Err(DomainError::ScopeInvalid));
    }
}
