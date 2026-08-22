//! Canonical representations and cryptographic envelope decoding.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{DomainError, DomainResult, ModelReference};

pub(super) fn canonicalize(value: &impl Serialize) -> DomainResult<Vec<u8>> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| DomainError::ManifestInvalid)
}

pub(super) fn verify_reference_hash(
    reference: &ModelReference,
    canonical_document: &[u8],
) -> DomainResult<()> {
    if hex_lower(&Sha256::digest(canonical_document)) != reference.manifest_sha256() {
        return Err(DomainError::ManifestInvalid);
    }
    Ok(())
}

pub(super) fn decode_signature(value: &str) -> DomainResult<[u8; 64]> {
    let encoded = value
        .strip_prefix("base64url:")
        .ok_or(DomainError::SignatureInvalid)?;
    URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| DomainError::SignatureInvalid)?
        .try_into()
        .map_err(|_| DomainError::SignatureInvalid)
}

pub(super) fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
