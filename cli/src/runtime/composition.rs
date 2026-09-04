use std::sync::Arc;

#[cfg(feature = "runtime")]
use synapseflow_adapter_in_memory::InMemoryAuditSink;
use synapseflow_application::GenerationOrchestrator;
#[cfg(feature = "runtime")]
use synapseflow_domain::ModelManifest;
use synapseflow_domain::{DomainError, DomainResult, ModelConfig, ModelReference};
use synapseflow_ports::AuditSink;
#[cfg(feature = "runtime")]
use synapseflow_ports::ShardedGenerationRuntime;

/// Builds the application-owned generation orchestrator for this CLI shell.
pub(crate) fn build_generation_orchestrator(
    reference: &ModelReference,
    config: ModelConfig,
) -> DomainResult<GenerationOrchestrator> {
    #[cfg(feature = "runtime")]
    {
        build_local_generation_orchestrator(
            reference,
            config,
            Arc::new(InMemoryAuditSink::default()),
        )
    }

    #[cfg(not(feature = "runtime"))]
    {
        let _ = reference;
        let _ = config;
        Err(DomainError::BackendUnavailable)
    }
}

/// Builds the node's shared application orchestrator with its durable audit sink.
pub(crate) fn build_node_generation_orchestrator(
    reference: &ModelReference,
    config: ModelConfig,
    audit: Arc<dyn AuditSink>,
) -> DomainResult<GenerationOrchestrator> {
    #[cfg(feature = "runtime")]
    {
        build_local_generation_orchestrator(reference, config, audit)
    }

    #[cfg(not(feature = "runtime"))]
    {
        let _ = reference;
        let _ = config;
        let _ = audit;
        Err(DomainError::BackendUnavailable)
    }
}

#[cfg(feature = "runtime")]
fn build_local_generation_orchestrator(
    reference: &ModelReference,
    config: ModelConfig,
    audit: Arc<dyn AuditSink>,
) -> DomainResult<GenerationOrchestrator> {
    use std::fs;

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
    let registry =
        ProvisionedManifestRegistry::new(trust_store, [(reference.clone(), manifest_document)])?;
    let mut artifacts =
        ContentAddressedArtifactStore::new(config.cache_directory, 2 * 1024 * 1024 * 1024)?;
    for artifact in &manifest.artifacts {
        artifacts
            .register_provisioned_source(artifact.uri.clone(), config.artifact_path.clone())?;
    }
    let backend = LlamaCppBackend::new()?;
    let sharded = (manifest.schema_version == 2)
        .then(|| build_loom_runtime(&manifest, audit.clone()))
        .transpose()?;
    Ok(GenerationOrchestrator::new(
        Arc::new(registry),
        Arc::new(artifacts),
        Arc::new(backend),
        sharded,
        audit,
    ))
}

#[cfg(feature = "runtime")]
fn build_loom_runtime(
    manifest: &ModelManifest,
    audit: Arc<dyn AuditSink>,
) -> DomainResult<Arc<dyn ShardedGenerationRuntime>> {
    use std::{collections::BTreeMap, sync::Arc};

    use synapseflow_adapter_in_memory::InMemoryPeerDirectory;
    use synapseflow_adapter_loom::{LoomBackend, LoomTokenizer};
    use synapseflow_adapter_loopback::LoopbackNetwork;
    use synapseflow_application::{LayerRangeShardedGenerationRuntime, SessionManager};
    use synapseflow_domain::execution::InFlightFrameLimit;
    use synapseflow_ports::{
        ShardAvailability, ShardExecutionBackend, WorkerCapability, WorkerHealth, WorkerId,
    };

    let plan = manifest
        .execution_plan
        .as_ref()
        .ok_or(DomainError::ShardPlanInvalid)?;
    let mut capabilities = Vec::new();
    let mut worker_ids = Vec::new();
    let mut backends = BTreeMap::<WorkerId, Arc<dyn ShardExecutionBackend>>::new();
    for shard in &plan.shards {
        for replica in 0..shard.minimum_replicas() {
            let worker = WorkerId::new(format!("loom-{}-{replica}", shard.id().as_str()))?;
            capabilities.push(WorkerCapability::new(
                worker.clone(),
                WorkerHealth::Healthy,
                vec![plan.strategy.clone()],
                vec![ShardAvailability {
                    model: manifest.reference.clone(),
                    shard: shard.id().clone(),
                }],
            )?);
            worker_ids.push(worker.clone());
            backends.insert(worker, Arc::new(LoomBackend::new()));
        }
    }
    let network = LoopbackNetwork::new(InFlightFrameLimit::new(4)?, worker_ids)?;
    Ok(Arc::new(LayerRangeShardedGenerationRuntime::new(
        Arc::new(InMemoryPeerDirectory::new(capabilities)),
        Arc::new(SessionManager::new(audit)),
        network.transport(),
        backends,
        Arc::new(LoomTokenizer::new()),
    )))
}
