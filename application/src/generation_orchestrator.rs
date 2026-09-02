use std::sync::Arc;

use synapseflow_domain::{
    DomainError, DomainResult, GenerationOutput, GenerationRequest, LOOM_RUNTIME_PROFILE,
    LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, ModelBackend, ModelRegistry, ShardedGenerationRuntime,
};

/// Selects a generation profile from a verified manifest and owns one request lifecycle.
pub struct GenerationOrchestrator {
    registry: Arc<dyn ModelRegistry>,
    artifacts: Arc<dyn ArtifactStore>,
    local: Arc<dyn ModelBackend>,
    sharded: Option<Arc<dyn ShardedGenerationRuntime>>,
    audit: Arc<dyn AuditSink>,
}

impl GenerationOrchestrator {
    pub fn new(
        registry: Arc<dyn ModelRegistry>,
        artifacts: Arc<dyn ArtifactStore>,
        local: Arc<dyn ModelBackend>,
        sharded: Option<Arc<dyn ShardedGenerationRuntime>>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            registry,
            artifacts,
            local,
            sharded,
            audit,
        }
    }

    pub fn generate(&self, request: GenerationRequest) -> DomainResult<GenerationOutput> {
        ensure_deadline(&request)?;
        self.audit.record(AuditEvent::GenerationStarted {
            model: request.model.clone(),
        })?;
        let result = self.generate_inner(&request);
        self.audit.record(match &result {
            Ok(output) => AuditEvent::GenerationCompleted {
                model: request.model.clone(),
                token_count: output.tokens.len(),
            },
            Err(_) => AuditEvent::GenerationFailed {
                model: request.model.clone(),
            },
        })?;
        result
    }

    fn generate_inner(&self, request: &GenerationRequest) -> DomainResult<GenerationOutput> {
        ensure_deadline(request)?;
        let manifest = self.registry.resolve(&request.model)?;
        ensure_deadline(request)?;
        let model = self.artifacts.acquire(&manifest)?;
        ensure_deadline(request)?;
        match manifest.schema_version {
            MANIFEST_SCHEMA_VERSION if manifest.supports_verified_local_inference() => {
                self.local.generate(&model, request)
            }
            LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION
                if manifest.execution_plan.is_some()
                    && manifest.runtime_profile.as_deref() == Some(LOOM_RUNTIME_PROFILE) =>
            {
                self.sharded
                    .as_ref()
                    .ok_or(DomainError::BackendUnavailable)?
                    .generate(&model, request)
            }
            _ => Err(DomainError::ManifestUnsupported),
        }
    }
}

fn ensure_deadline(request: &GenerationRequest) -> DomainResult<()> {
    if request.deadline_expired() {
        return Err(DomainError::DeadlineExceeded);
    }
    Ok(())
}
