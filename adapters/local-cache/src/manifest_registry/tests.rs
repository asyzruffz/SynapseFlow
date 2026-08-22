use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use synapseflow_domain::{DomainError, ModelReference, TrustStore, TrustedPublisher};
use synapseflow_ports::ModelRegistry;

use super::ProvisionedManifestRegistry;

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[9; 32])
}

fn trust_store() -> TrustStore {
    let key = signing_key().verifying_key();
    TrustStore::new([TrustedPublisher::new(
        "ed25519:registry-test".to_owned(),
        &URL_SAFE_NO_PAD.encode(key.to_bytes()),
    )
    .expect("test key should be valid")])
    .expect("test trust store should be valid")
}

fn unsigned_manifest() -> Value {
    json!({
        "schema_version": 1,
        "model_id": "tinyllama-chat",
        "model_version": "test",
        "format": "gguf",
        "architecture": "llama",
        "quantization": "Q5_K_M",
        "tokenizer": { "kind": "embedded", "model": "llama" },
        "artifacts": [{
            "artifact_id": "weights",
            "uri": "https://fixtures.example/weights.gguf",
            "content_sha256": format!("sha256:{}", "a".repeat(64)),
            "size_bytes": 1
        }],
        "publisher_key_id": "ed25519:registry-test",
        "license": "Apache-2.0",
        "provenance": "fixture:test"
    })
}

fn signed_document(signature: Option<String>) -> (ModelReference, Vec<u8>) {
    let mut document = unsigned_manifest();
    let signature = signature.unwrap_or_else(|| {
        let canonical = serde_json_canonicalizer::to_vec(&document)
            .expect("test unsigned manifest should canonicalize");
        format!(
            "base64url:{}",
            URL_SAFE_NO_PAD.encode(signing_key().sign(&canonical).to_bytes())
        )
    });
    document
        .as_object_mut()
        .expect("test manifest should be an object")
        .insert("signature".to_owned(), Value::String(signature));
    let canonical =
        serde_json_canonicalizer::to_vec(&document).expect("test manifest should canonicalize");
    let reference = ModelReference::parse(format!(
        "registry://fixtures/manifest@sha256:{}",
        hex(&Sha256::digest(&canonical))
    ))
    .expect("test reference should be valid");
    (reference, canonical)
}

#[test]
fn resolves_only_a_provisioned_and_validly_signed_manifest() {
    let (reference, document) = signed_document(None);
    let registry = ProvisionedManifestRegistry::new(trust_store(), [(reference.clone(), document)])
        .expect("registry configuration should be valid");

    let manifest = registry
        .resolve(&reference)
        .expect("provisioned signed manifest should resolve");

    assert_eq!(manifest.reference, reference);
    assert!(manifest.supports_verified_local_inference());
}

#[test]
fn invalid_signature_is_a_typed_failure_before_cache_use() {
    let (reference, document) = signed_document(Some(format!(
        "base64url:{}",
        URL_SAFE_NO_PAD.encode([0_u8; 64])
    )));
    let registry = ProvisionedManifestRegistry::new(trust_store(), [(reference.clone(), document)])
        .expect("registry configuration should be valid");

    assert!(matches!(
        registry.resolve(&reference),
        Err(DomainError::SignatureInvalid)
    ));
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
