use synapseflow_domain::{AuthenticatedPrincipal, DomainError, DomainResult};

/// Opaque bearer credential received at the node boundary.
///
/// It is intentionally not `Debug` or `Clone`, preventing accidental logging
/// or retention by application code.
pub struct BearerCredential<'a>(&'a str);

impl<'a> BearerCredential<'a> {
    pub fn new(value: &'a str) -> DomainResult<Self> {
        if value.is_empty() {
            Err(DomainError::AuthenticationInvalid)
        } else {
            Ok(Self(value))
        }
    }

    pub fn expose_to_verifier(&self) -> &str {
        self.0
    }
}

/// Verifies a caller credential and returns only domain-safe identity data.
pub trait IdentityVerifier: Send + Sync {
    fn verify(&self, credential: BearerCredential<'_>) -> DomainResult<AuthenticatedPrincipal>;
}

#[cfg(test)]
mod tests {
    use super::BearerCredential;
    use synapseflow_domain::DomainError;

    #[test]
    fn rejects_an_empty_credential_before_identity_verification() {
        assert!(matches!(
            BearerCredential::new(""),
            Err(DomainError::AuthenticationInvalid)
        ));
    }
}
