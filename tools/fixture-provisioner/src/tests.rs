use std::path::PathBuf;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::SigningKey;
use synapseflow_domain::{ModelManifest, ModelReference, TrustStore, TrustedPublisher};

use crate::{
    artifact::ArtifactFingerprint, manifest::SignedManifest, request::ProvisioningRequest,
};

#[test]
fn generated_manifest_round_trips_through_the_domain_verifier() {
    let key_path = unique_temporary_path("fixture-signing-key");
    let output_path = unique_temporary_path("fixture-manifest");
    let signing_key = SigningKey::from_bytes(&[11; 32]);
    std::fs::write(&key_path, URL_SAFE_NO_PAD.encode(signing_key.to_bytes()))
        .expect("temporary signing key should be writable");

    let request = ProvisioningRequest {
        artifact_path: PathBuf::from("fixture.gguf"),
        artifact_uri: "https://fixtures.example/tinyllama.gguf".to_owned(),
        signing_key_path: key_path.clone(),
        output_path,
        model_id: "tinyllama-chat".to_owned(),
        model_version: "1.1b-q5km-2026-08-22".to_owned(),
        publisher_key_id: "ed25519:fixture-test".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "fixture:tinyllama".to_owned(),
    };
    let artifact = ArtifactFingerprint {
        size_bytes: 12,
        content_sha256: "a".repeat(64),
    };

    let signed = SignedManifest::create(&request, &artifact)
        .expect("the provisioned manifest should self-verify");
    let reference = ModelReference::parse(signed.reference.clone())
        .expect("the generated reference should be valid");
    let publisher = TrustedPublisher::new(request.publisher_key_id, &signed.public_key_base64url)
        .expect("the generated public key should be valid");
    let trust_store = TrustStore::new([publisher]).expect("the trust store should be valid");

    let manifest = ModelManifest::parse_and_verify(reference, &signed.document, &trust_store)
        .expect("the generated manifest should verify");
    assert_eq!(manifest.artifacts[0].size_bytes, 12);

    std::fs::remove_file(key_path).expect("temporary signing key should be removable");
}

fn unique_temporary_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()))
}
