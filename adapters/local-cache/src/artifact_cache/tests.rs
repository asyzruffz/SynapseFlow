use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use sha2::{Digest, Sha256};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{ArtifactStore, CacheEntryState};

use super::ContentAddressedArtifactStore;

static TEMPORARY_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new(name: &str) -> Self {
        let id = TEMPORARY_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "synapseflow-local-cache-{name}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test directory should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest(content: &[u8], uri: &str, name: &str) -> ModelManifest {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/{name}@sha256:{}",
        "b".repeat(64)
    ))
    .expect("test reference should be valid");
    ModelManifest {
        reference,
        schema_version: MANIFEST_SCHEMA_VERSION,
        model_id: "tinyllama".to_owned(),
        model_version: name.to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights".to_owned()).expect("test ID should be valid"),
            uri: uri.to_owned(),
            content_sha256: format!("sha256:{}", hex(&Sha256::digest(content))),
            size_bytes: content.len() as u64,
        }],
        publisher_key_id: "ed25519:cache-test".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "fixture:test".to_owned(),
        execution_plan: None,
        runtime_profile: None,
    }
}

fn configured_store(
    root: &Path,
    manifest: &ModelManifest,
    source: &Path,
) -> ContentAddressedArtifactStore {
    let mut store = ContentAddressedArtifactStore::new(root.to_path_buf(), 1024 * 1024)
        .expect("cache should be configured");
    store
        .register_provisioned_source(manifest.artifacts[0].uri.clone(), source.to_path_buf())
        .expect("source should be permitted");
    store
}

#[test]
fn promotes_a_verified_source_then_serves_a_cache_hit_without_the_source() {
    let temporary = TemporaryDirectory::new("hit");
    let source = temporary.path().join("source.gguf");
    let content = b"verified test artifact";
    fs::write(&source, content).expect("source should be written");
    let manifest = manifest(content, "https://fixtures.example/model.gguf", "hit");
    let store = configured_store(temporary.path(), &manifest, &source);

    store
        .acquire(&manifest)
        .expect("first acquisition should verify");
    fs::remove_file(&source).expect("source should be removable after promotion");
    store
        .acquire(&manifest)
        .expect("cache hit should not need source");

    let inspection = store.inspect(&manifest).expect("cache should inspect");
    assert_eq!(inspection.artifacts[0].state, CacheEntryState::Cached);
    let hash = manifest.artifacts[0]
        .content_sha256
        .strip_prefix("sha256:")
        .expect("test hash should have prefix");
    assert!(temporary
        .path()
        .join("metadata")
        .join(format!("{hash}.meta"))
        .exists());
}

#[test]
fn rejects_mismatched_or_missing_sources_without_promoting_staging() {
    let temporary = TemporaryDirectory::new("mismatch");
    let source = temporary.path().join("source.gguf");
    fs::write(&source, b"tampered").expect("source should be written");
    let manifest = manifest(
        b"expected",
        "https://fixtures.example/model.gguf",
        "mismatch",
    );
    let store = configured_store(temporary.path(), &manifest, &source);

    assert!(matches!(
        store.acquire(&manifest),
        Err(DomainError::ArtifactIntegrity)
    ));
    assert!(fs::read_dir(temporary.path().join("staging"))
        .expect("staging should remain readable")
        .next()
        .is_none());

    fs::remove_file(&source).expect("source should be removable");
    assert!(matches!(
        store.acquire(&manifest),
        Err(DomainError::ArtifactUnavailable)
    ));
}

#[test]
fn rejects_an_unprovisioned_artifact_uri() {
    let temporary = TemporaryDirectory::new("source-policy");
    let manifest = manifest(
        b"source",
        "https://fixtures.example/model.gguf",
        "source-policy",
    );
    let store = ContentAddressedArtifactStore::new(temporary.path().to_path_buf(), 1024 * 1024)
        .expect("cache should be configured");

    assert!(matches!(
        store.acquire(&manifest),
        Err(DomainError::DisallowedSource)
    ));
}

#[test]
fn evicts_inactive_objects_when_one_model_is_retained() {
    let temporary = TemporaryDirectory::new("eviction");
    let first_source = temporary.path().join("first.gguf");
    let second_source = temporary.path().join("second.gguf");
    fs::write(&first_source, b"first").expect("first source should be written");
    fs::write(&second_source, b"second").expect("second source should be written");
    let first = manifest(b"first", "https://fixtures.example/first.gguf", "first");
    let second = manifest(b"second", "https://fixtures.example/second.gguf", "second");
    let mut store = ContentAddressedArtifactStore::new(temporary.path().to_path_buf(), 1024 * 1024)
        .expect("cache should be configured");
    store
        .register_provisioned_source(first.artifacts[0].uri.clone(), first_source)
        .expect("first source should be allowed");
    store
        .register_provisioned_source(second.artifacts[0].uri.clone(), second_source)
        .expect("second source should be allowed");
    store.acquire(&first).expect("first model should cache");
    store.acquire(&second).expect("second model should cache");

    store
        .cleanup_except(&second)
        .expect("cleanup should succeed");

    assert_eq!(
        store
            .inspect(&first)
            .expect("first cache should inspect")
            .artifacts[0]
            .state,
        CacheEntryState::Missing
    );
    assert_eq!(
        store
            .inspect(&second)
            .expect("second cache should inspect")
            .artifacts[0]
            .state,
        CacheEntryState::Cached
    );
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
