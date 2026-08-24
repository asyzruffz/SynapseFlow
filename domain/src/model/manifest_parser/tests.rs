use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

use super::{
    integrity::{canonicalize, hex_lower},
    schema::{
        ArtifactWire, ExecutionWire, ManifestWire, ShardWire, ShardedManifestWire, TokenizerWire,
        UnsignedManifest,
    },
};
use crate::{
    DomainError, ModelManifest, ModelReference, TrustStore, TrustedPublisher, LOOM_RUNTIME_PROFILE,
    LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};

const KEY_ID: &str = "ed25519:manifest-test";
const ARTIFACT_HASH: &str =
    "sha256:7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed";
const SHARDED_CANONICAL_VECTOR: &[u8] = br#"{"architecture":"llama","artifacts":[{"artifact_id":"weights","content_sha256":"sha256:7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed","size_bytes":782052992,"uri":"https://fixtures.example/models/tinyllama.Q5_K_M.gguf"}],"execution":{"layer_count":22,"runtime_profile":"synapseflow-loom-llama-v1","shards":[{"artifact_id":"weights","layer_end_exclusive":11,"layer_start":0,"minimum_replicas":1,"shard_id":"first"},{"artifact_id":"weights","layer_end_exclusive":22,"layer_start":11,"minimum_replicas":1,"shard_id":"second"}],"strategy":"layer_range_v1"},"format":"gguf","license":"Apache-2.0","model_id":"tinyllama-chat","model_version":"1.1b-q5km-loopback-v1","provenance":"fixture:tinyllama","publisher_key_id":"ed25519:manifest-test","quantization":"Q5_K_M","schema_version":2,"signature":"base64url:VXxg0UiIrTX-w9Mwuxi0VPJMzJg4MNyub8Z_GChygYdnHyt9KqibO_J2-OcIk7MEJYQVeJTw2iW78XfSLVpMDw","tokenizer":{"kind":"embedded","model":"llama"}}"#;

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

fn sharded_wire() -> ShardedManifestWire {
    ShardedManifestWire {
        schema_version: LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
        model_id: "tinyllama-chat".to_owned(),
        model_version: "1.1b-q5km-loopback-v1".to_owned(),
        format: "gguf".to_owned(),
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerWire {
            kind: "embedded".to_owned(),
            model: "llama".to_owned(),
        },
        artifacts: unsigned_wire().artifacts,
        execution: ExecutionWire {
            strategy: "layer_range_v1".to_owned(),
            runtime_profile: LOOM_RUNTIME_PROFILE.to_owned(),
            layer_count: 22,
            shards: vec![
                ShardWire {
                    shard_id: "first".to_owned(),
                    artifact_id: "weights".to_owned(),
                    layer_start: 0,
                    layer_end_exclusive: 11,
                    minimum_replicas: 1,
                },
                ShardWire {
                    shard_id: "second".to_owned(),
                    artifact_id: "weights".to_owned(),
                    layer_start: 11,
                    layer_end_exclusive: 22,
                    minimum_replicas: 1,
                },
            ],
        },
        publisher_key_id: KEY_ID.to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "fixture:tinyllama".to_owned(),
        signature: String::new(),
    }
}

fn sign_sharded(mut wire: ShardedManifestWire) -> ShardedManifestWire {
    let canonical_unsigned = canonicalize(&UnsignedManifest::from(&wire))
        .expect("test sharded manifest should canonicalize");
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

fn reference_for_sharded(wire: &ShardedManifestWire) -> ModelReference {
    let canonical = canonicalize(wire).expect("test sharded manifest should canonicalize");
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

fn signed_sharded_document() -> (ModelReference, Vec<u8>) {
    let wire = sign_sharded(sharded_wire());
    let canonical = canonicalize(&wire).expect("test sharded manifest should canonicalize");
    let reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        hex_lower(&Sha256::digest(canonical))
    ))
    .expect("test reference should be valid");
    let document = serde_json::to_vec(&wire).expect("test sharded manifest should serialize");
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
fn verifies_a_canonical_signed_sharded_manifest() {
    let (reference, document) = signed_sharded_document();

    let manifest = ModelManifest::parse_and_verify(reference, &document, &trust_store())
        .expect("signed sharded fixture manifest should verify");

    let plan = manifest
        .execution_plan
        .expect("sharded manifest should retain an execution plan");
    assert_eq!(plan.total_layers(), 22);
    assert_eq!(plan.shards[0].artifact_id().as_str(), "weights");
}

#[test]
fn canonical_signed_sharded_manifest_matches_the_golden_vector() {
    let wire = sign_sharded(sharded_wire());
    let canonical = canonicalize(&wire).expect("test sharded manifest should canonicalize");

    assert_eq!(canonical, SHARDED_CANONICAL_VECTOR);
}

#[test]
fn rejects_shards_with_unknown_artifacts_or_gaps() {
    let mut unknown_artifact = sharded_wire();
    unknown_artifact.execution.shards[1].artifact_id = "missing".to_owned();
    let unknown_artifact = sign_sharded(unknown_artifact);
    let unknown_canonical =
        canonicalize(&unknown_artifact).expect("test sharded manifest should canonicalize");
    let unknown_reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        hex_lower(&Sha256::digest(unknown_canonical))
    ))
    .expect("test reference should be valid");
    let unknown_document =
        serde_json::to_vec(&unknown_artifact).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(unknown_reference, &unknown_document, &trust_store()),
        Err(DomainError::ManifestInvalid)
    ));

    let mut gapped = sharded_wire();
    gapped.execution.shards[1].layer_start = 12;
    let gapped = sign_sharded(gapped);
    let gapped_canonical =
        canonicalize(&gapped).expect("test sharded manifest should canonicalize");
    let gapped_reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        hex_lower(&Sha256::digest(gapped_canonical))
    ))
    .expect("test reference should be valid");
    let gapped_document =
        serde_json::to_vec(&gapped).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(gapped_reference, &gapped_document, &trust_store()),
        Err(DomainError::ShardPlanInvalid)
    ));
}

#[test]
fn rejects_incomplete_reordered_altered_and_incompatible_sharded_manifests() {
    let mut incomplete = sharded_wire();
    incomplete.execution.layer_count = 23;
    let incomplete = sign_sharded(incomplete);
    let incomplete_reference = reference_for_sharded(&incomplete);
    let incomplete_document =
        serde_json::to_vec(&incomplete).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(incomplete_reference, &incomplete_document, &trust_store()),
        Err(DomainError::ShardPlanInvalid)
    ));

    let mut reordered = sharded_wire();
    reordered.execution.shards.swap(0, 1);
    let reordered = sign_sharded(reordered);
    let reordered_reference = reference_for_sharded(&reordered);
    let reordered_document =
        serde_json::to_vec(&reordered).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(reordered_reference, &reordered_document, &trust_store()),
        Err(DomainError::ShardPlanInvalid)
    ));

    let mut overlapping = sharded_wire();
    overlapping.execution.shards[1].layer_start = 10;
    let overlapping = sign_sharded(overlapping);
    let overlapping_reference = reference_for_sharded(&overlapping);
    let overlapping_document =
        serde_json::to_vec(&overlapping).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(
            overlapping_reference,
            &overlapping_document,
            &trust_store()
        ),
        Err(DomainError::ShardPlanInvalid)
    ));

    let signed = sign_sharded(sharded_wire());
    let mut altered = signed.clone();
    altered.model_version = "altered".to_owned();
    let altered_reference = reference_for_sharded(&altered);
    let altered_document =
        serde_json::to_vec(&altered).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(altered_reference, &altered_document, &trust_store()),
        Err(DomainError::SignatureInvalid)
    ));

    let mut incompatible = sharded_wire();
    incompatible.execution.runtime_profile = "unknown-runtime".to_owned();
    let incompatible = sign_sharded(incompatible);
    let incompatible_reference = reference_for_sharded(&incompatible);
    let incompatible_document =
        serde_json::to_vec(&incompatible).expect("test sharded manifest should serialize");
    assert!(matches!(
        ModelManifest::parse_and_verify(
            incompatible_reference,
            &incompatible_document,
            &trust_store()
        ),
        Err(DomainError::ManifestUnsupported)
    ));
}

#[test]
fn rejects_a_malformed_sharded_document_before_signature_handling() {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test reference should be valid");
    let malformed = br#"{"schema_version":2,"execution":{"strategy":"layer_range_v1"}}"#;

    assert!(matches!(
        ModelManifest::parse_and_verify(reference, malformed, &trust_store()),
        Err(DomainError::ManifestInvalid)
    ));
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
