use std::{
    fs,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use synapseflow_adapter_in_memory::InMemoryAuditSink;
use synapseflow_adapter_local_cache::{ContentAddressedArtifactStore, ProvisionedManifestRegistry};
use synapseflow_application::GenerationOrchestrator;
use synapseflow_domain::{GenerationRequest, ModelManifest};

use super::{config::FixtureConfig, vector::ReferenceVector};
use crate::LlamaCppBackend;

static CACHE_SUFFIX: AtomicU64 = AtomicU64::new(0);

pub(super) fn assert_reference_output() -> Result<(), String> {
    let config = FixtureConfig::from_environment()?;
    let manifest = ModelManifest::parse_and_verify(
        config.reference.clone(),
        &config.manifest_document,
        &config.trust_store,
    )
    .map_err(|error| format!("provisioned fixture manifest is invalid: {error}"))?;
    let artifact = manifest
        .artifacts
        .first()
        .ok_or_else(|| "provisioned fixture manifest contains no artifact".to_owned())?;
    let cache_directory = TemporaryCache::new();
    let mut cache = ContentAddressedArtifactStore::new(
        cache_directory.path().to_path_buf(),
        artifact.size_bytes,
    )
    .map_err(|error| format!("cannot create fixture cache: {error}"))?;
    cache
        .register_provisioned_source(artifact.uri.clone(), config.artifact_path.clone())
        .map_err(|error| format!("cannot register fixture artifact: {error}"))?;
    let cache = Arc::new(cache);
    let registry = Arc::new(
        ProvisionedManifestRegistry::new(
            config.trust_store.clone(),
            [(config.reference.clone(), config.manifest_document.clone())],
        )
        .map_err(|error| format!("cannot provision fixture manifest registry: {error}"))?,
    );
    let backend = Arc::new(LlamaCppBackend::new().map_err(|error| {
        format!("cannot initialize the CPU llama.cpp runtime for the fixture: {error}")
    })?);
    let orchestrator = GenerationOrchestrator::new(
        registry,
        cache,
        backend,
        None,
        Arc::new(InMemoryAuditSink::default()),
    );
    let request = GenerationRequest::new(
        config.reference.clone(),
        ReferenceVector::prompt().to_owned(),
        ReferenceVector::policy()?,
    )
    .map_err(|error| format!("cannot create fixed fixture request: {error}"))?;
    let output = orchestrator
        .generate(request)
        .map_err(|error| format!("fixture generation failed: {error}"))?;

    if let Some(expected) = config.expected_vector {
        return expected.assert_matches(
            FixtureConfig::fixture_id(),
            &config.reference,
            &config.llama_cpp_revision,
            &output,
        );
    }

    let candidate = ReferenceVector::candidate(
        FixtureConfig::fixture_id(),
        &config.reference,
        config.llama_cpp_revision,
        &output,
    );
    let candidate_path = config
        .candidate_vector_path
        .ok_or_else(|| "candidate vector output path was not supplied".to_owned())?;
    candidate.write_new(&candidate_path)?;
    Err(format!(
        "candidate vector written to {}; review and reproduce it on the other Tier-1 platform before using it as SYNAPSEFLOW_REFERENCE_VECTOR",
        candidate_path.display()
    ))
}

struct TemporaryCache {
    path: std::path::PathBuf,
}

impl TemporaryCache {
    fn new() -> Self {
        let suffix = CACHE_SUFFIX.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "synapseflow-fixture-reference-{}-{suffix}",
            std::process::id()
        ));
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TemporaryCache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
