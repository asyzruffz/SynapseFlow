//! Bounded parsing, signature verification, and validation for model manifests.

mod integrity;
mod schema;
#[cfg(test)]
mod tests;
mod validation;

use crate::{
    DomainError, DomainResult, ModelManifest, ModelReference, TrustStore, MAX_MANIFEST_BYTES,
};

use self::{
    integrity::{canonicalize, decode_signature, verify_reference_hash},
    schema::{ManifestDocument, UnsignedManifest},
    validation::into_manifest,
};

/// Parses one bounded JSON document, verifies its signature, and validates its local-inference shape.
pub(super) fn parse_and_verify(
    reference: ModelReference,
    document: &[u8],
    trust_store: &TrustStore,
) -> DomainResult<ModelManifest> {
    if document.is_empty() || document.len() > MAX_MANIFEST_BYTES {
        return Err(DomainError::ManifestInvalid);
    }

    let wire: ManifestDocument =
        serde_json::from_slice(document).map_err(|_| DomainError::ManifestInvalid)?;
    verify_reference_hash(&reference, &canonicalize(&wire)?)?;

    let signature = decode_signature(wire.signature())?;
    trust_store.verify_strict(
        wire.publisher_key_id(),
        &canonicalize(&UnsignedManifest::from(&wire))?,
        &signature,
    )?;

    into_manifest(reference, wire)
}
