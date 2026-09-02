use synapseflow_application::GenerationService;
use synapseflow_domain::{DomainError, DomainResult, ModelConfig, ModelReference};

/// Builds the verified-local service used by this CLI shell.
pub(crate) fn build_generation_service(
    reference: &ModelReference,
    config: ModelConfig,
) -> DomainResult<GenerationService> {
    #[cfg(feature = "runtime")]
    {
        build_local_generation_service(reference, config)
    }

    #[cfg(not(feature = "runtime"))]
    {
        let _ = reference;
        let _ = config;
        Err(DomainError::BackendUnavailable)
    }
}

#[cfg(feature = "runtime")]
fn build_local_generation_service(
    reference: &ModelReference,
    config: ModelConfig,
) -> DomainResult<GenerationService> {
    use std::{fs, sync::Arc};

    use synapseflow_adapter_in_memory::InMemoryAuditSink;
    use synapseflow_adapter_llama_cpp::LlamaCppBackend;
    use synapseflow_adapter_local_cache::{
        ContentAddressedArtifactStore, ProvisionedManifestRegistry,
    };
    use synapseflow_domain::{ModelManifest, TrustStore, TrustedPublisher};

    config.validate()?;
    let manifest_document =
        fs::read(config.manifest_path).map_err(|_| DomainError::ManifestUnavailable)?;
    let publisher = TrustedPublisher::new(
        "ed25519:synapseflow-fixture-2026-08".to_owned(),
        &config.publisher_public_key,
    )?;
    let trust_store = TrustStore::new([publisher])?;
    let manifest =
        ModelManifest::parse_and_verify(reference.clone(), &manifest_document, &trust_store)?;
    let artifact_uri = manifest
        .artifacts
        .first()
        .ok_or(DomainError::ManifestUnsupported)?
        .uri
        .clone();
    let registry =
        ProvisionedManifestRegistry::new(trust_store, [(reference.clone(), manifest_document)])?;
    let mut artifacts =
        ContentAddressedArtifactStore::new(config.cache_directory, 2 * 1024 * 1024 * 1024)?;
    artifacts.register_provisioned_source(artifact_uri, config.artifact_path)?;
    let backend = LlamaCppBackend::new()?;
    Ok(GenerationService::new(
        Arc::new(registry),
        Arc::new(artifacts),
        Arc::new(backend),
        Arc::new(InMemoryAuditSink::default()),
    ))
}
