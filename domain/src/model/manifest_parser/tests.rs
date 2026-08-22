use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

use super::{
    integrity::{canonicalize, hex_lower},
    schema::{ArtifactWire, ManifestWire, TokenizerWire, UnsignedManifest},
};
use crate::{
    DomainError, ModelManifest, ModelReference, TrustStore, TrustedPublisher,
    MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};

const KEY_ID: &str = "ed25519:manifest-test";
const ARTIFACT_HASH: &str =
    "sha256:7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed";

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}

fn trust_store() -> TrustStore {
    let key = signing_key().verifying_key();
    TrustStore::new([TrustedPublisher::new(
        KEY_ID.to_owned(),
        &URL_SAFE_NO_PAD.encode(key.to_bytes()),
    )
    .expect("test public key should be valid")])
    .expect("test trust store should be valid")
}

fn unsigned_wire() -> ManifestWire {
    ManifestWire {
        schema_version: MANIFEST_SCHEMA_VERSION,
        model_id: "tinyllama-chat".to_owned(),
        model_version: "1.1b-q5km-2026-08-22".to_owned(),
        format: "gguf".to_owned(),
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerWire {
            kind: "embedded".to_owned(),
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactWire {
            artifact_id: "weights".to_owned(),
            uri: "https://fixtures.example/models/tinyllama.Q5_K_M.gguf".to_owned(),
            content_sha256: ARTIFACT_HASH.to_owned(),
            size_bytes: 782_052_992,
        }],
        publisher_key_id: KEY_ID.to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "fixture:tinyllama".to_owned(),
        signature: String::new(),
    }
}

fn sign(mut wire: ManifestWire) -> ManifestWire {
    let canonical_unsigned =
        canonicalize(&UnsignedManifest::from(&wire)).expect("test manifest should canonicalize");
    wire.signature = format!(
        "base64url:{}",
        URL_SAFE_NO_PAD.encode(signing_key().sign(&canonical_unsigned).to_bytes())
    );
    wire
}

fn reference_for(wire: &ManifestWire) -> ModelReference {
    let canonical = canonicalize(wire).expect("test manifest should canonicalize");
    ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        hex_lower(&Sha256::digest(canonical))
    ))
    .expect("test reference should be valid")
}

fn signed_document() -> (ModelReference, Vec<u8>) {
    let wire = sign(unsigned_wire());
    let reference = reference_for(&wire);
    let document = serde_json::to_vec(&wire).expect("test manifest should serialize");
    (reference, document)
}

#[test]
fn verifies_a_canonical_signed_manifest() {
    let (reference, document) = signed_document();

    let manifest = ModelManifest::parse_and_verify(reference, &document, &trust_store())
        .expect("signed fixture manifest should verify");

    assert!(manifest.supports_verified_local_inference());
    assert_eq!(manifest.artifacts[0].content_sha256, ARTIFACT_HASH);
}

#[test]
fn canonical_unsigned_manifest_matches_the_golden_vector() {
    let canonical = canonicalize(&UnsignedManifest::from(&unsigned_wire()))
        .expect("test manifest should canonicalize");

    assert_eq!(
        canonical,
        br#"{"architecture":"llama","artifacts":[{"artifact_id":"weights","content_sha256":"sha256:7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed","size_bytes":782052992,"uri":"https://fixtures.example/models/tinyllama.Q5_K_M.gguf"}],"format":"gguf","license":"Apache-2.0","model_id":"tinyllama-chat","model_version":"1.1b-q5km-2026-08-22","provenance":"fixture:tinyllama","publisher_key_id":"ed25519:manifest-test","quantization":"Q5_K_M","schema_version":1,"tokenizer":{"kind":"embedded","model":"llama"}}"#
    );
}

#[test]
fn rejects_an_unknown_key() {
    let (reference, document) = signed_document();

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, &document, &TrustStore::default()),
        Err(DomainError::PublisherUntrusted)
    ));
}

#[test]
fn rejects_a_tampered_signature_after_reference_validation() {
    let mut wire = sign(unsigned_wire());
    wire.signature = format!("base64url:{}", URL_SAFE_NO_PAD.encode([0_u8; 64]));
    let reference = reference_for(&wire);
    let document = serde_json::to_vec(&wire).expect("test manifest should serialize");

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, &document, &trust_store()),
        Err(DomainError::SignatureInvalid)
    ));
}

#[test]
fn rejects_an_unsigned_manifest() {
    let wire = unsigned_wire();
    let reference = reference_for(&wire);
    let document = serde_json::to_vec(&wire).expect("test manifest should serialize");

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, &document, &trust_store()),
        Err(DomainError::SignatureInvalid)
    ));
}

#[test]
fn rejects_unsupported_schema_after_signature_verification() {
    let mut wire = unsigned_wire();
    wire.schema_version = MANIFEST_SCHEMA_VERSION + 1;
    let wire = sign(wire);
    let reference = reference_for(&wire);
    let document = serde_json::to_vec(&wire).expect("test manifest should serialize");

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, &document, &trust_store()),
        Err(DomainError::ManifestUnsupported)
    ));
}

#[test]
fn rejects_an_incompatible_manifest_after_signature_verification() {
    let mut wire = unsigned_wire();
    wire.quantization = "Q4_K_M".to_owned();
    let wire = sign(wire);
    let reference = reference_for(&wire);
    let document = serde_json::to_vec(&wire).expect("test manifest should serialize");

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, &document, &trust_store()),
        Err(DomainError::ManifestUnsupported)
    ));
}

#[test]
fn rejects_unknown_fields_and_oversized_documents() {
    let unknown_field = br#"{"unexpected":true}"#;
    let reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test reference should be valid");

    assert!(matches!(
        ModelManifest::parse_and_verify(reference.clone(), unknown_field, &trust_store()),
        Err(DomainError::ManifestInvalid)
    ));
    assert!(matches!(
        ModelManifest::parse_and_verify(
            reference,
            &vec![b' '; MAX_MANIFEST_BYTES + 1],
            &trust_store()
        ),
        Err(DomainError::ManifestInvalid)
    ));
}
