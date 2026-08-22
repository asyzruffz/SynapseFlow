//! Environment-specific trusted-key lookup and strict signature verification.

use std::collections::BTreeMap;

use ed25519_dalek::{Signature, VerifyingKey};

use crate::{DomainError, DomainResult};

use super::TrustedPublisher;

/// The configured, environment-specific set of publisher keys.
#[derive(Clone, Debug, Default)]
pub struct TrustStore {
    publishers: BTreeMap<String, TrustedPublisher>,
}

impl TrustStore {
    pub fn new(publishers: impl IntoIterator<Item = TrustedPublisher>) -> DomainResult<Self> {
        let mut store = Self::default();
        for publisher in publishers {
            store.insert(publisher)?;
        }
        Ok(store)
    }

    pub fn insert(&mut self, publisher: TrustedPublisher) -> DomainResult<()> {
        if self
            .publishers
            .insert(publisher.key_id.clone(), publisher)
            .is_some()
        {
            return Err(DomainError::PublisherUntrusted);
        }
        Ok(())
    }

    pub(crate) fn verify_strict(
        &self,
        key_id: &str,
        message: &[u8],
        signature: &[u8; 64],
    ) -> DomainResult<()> {
        let publisher = self
            .publishers
            .get(key_id)
            .ok_or(DomainError::PublisherUntrusted)?;
        let key = VerifyingKey::from_bytes(&publisher.public_key)
            .map_err(|_| DomainError::PublisherUntrusted)?;
        let signature = Signature::from_bytes(signature);
        key.verify_strict(message, &signature)
            .map_err(|_| DomainError::SignatureInvalid)
    }
}
