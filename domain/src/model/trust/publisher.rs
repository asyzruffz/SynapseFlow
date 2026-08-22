//! Validation and storage for a single trusted Ed25519 publisher key.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::VerifyingKey;

use crate::{DomainError, DomainResult};

/// One configured publisher verification key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedPublisher {
    pub key_id: String,
    pub(super) public_key: [u8; 32],
}

impl TrustedPublisher {
    /// Decodes a base64url Ed25519 public key after validating its key identifier.
    pub fn new(key_id: String, public_key_base64url: &str) -> DomainResult<Self> {
        if !key_id.starts_with("ed25519:") || key_id.len() <= "ed25519:".len() {
            return Err(DomainError::PublisherUntrusted);
        }

        let decoded = URL_SAFE_NO_PAD
            .decode(public_key_base64url)
            .map_err(|_| DomainError::PublisherUntrusted)?;
        let public_key: [u8; 32] = decoded
            .try_into()
            .map_err(|_| DomainError::PublisherUntrusted)?;
        VerifyingKey::from_bytes(&public_key).map_err(|_| DomainError::PublisherUntrusted)?;

        Ok(Self { key_id, public_key })
    }
}
