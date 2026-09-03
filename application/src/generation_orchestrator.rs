use std::sync::Arc;

use synapseflow_domain::{
    DomainError, DomainResult, GenerationEvent, GenerationOutput, GenerationRequest,
    GenerationTerminal, LOOM_RUNTIME_PROFILE, LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
    MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, ExecutionCancellation, GenerationEventSink, ModelBackend,
    ModelRegistry, NeverCancelled, ShardedGenerationRuntime,
};

use crate::live_generation::{OutputCollector, TokenEventForwarder};

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
        let mut collector = OutputCollector::new();
        self.generate_live(request, &NeverCancelled, &mut collector)?;
        collector.into_output()
    }

    /// Executes a request through the selected profile and emits ordered public events.
    pub fn generate_live(
        &self,
        request: GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<GenerationTerminal> {
        if cancellation.is_cancelled() {
            events.emit(GenerationEvent::Cancelled)?;
            return Ok(GenerationTerminal::Cancelled);
        }
        let result = self.generate_until_terminal(request, cancellation, events);
        match result {
            Ok(terminal) => {
                events.emit(match terminal {
                    GenerationTerminal::Completed { token_count } => {
                        GenerationEvent::Completed { token_count }
                    }
                    GenerationTerminal::Cancelled => GenerationEvent::Cancelled,
                })?;
                Ok(terminal)
            }
            Err(error) => {
                events.emit(GenerationEvent::Failed { code: error.code() })?;
                Err(error)
            }
        }
    }

    /// Generates only ordered tokens. The caller owns durable terminal transition and delivery.
    pub(crate) fn generate_until_terminal(
        &self,
        request: GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<GenerationTerminal> {
        if cancellation.is_cancelled() {
            return Ok(GenerationTerminal::Cancelled);
        }
        ensure_deadline(&request)?;
        self.audit.record(AuditEvent::GenerationStarted {
            model: request.model.clone(),
        })?;
        let result = self.generate_inner(&request, cancellation, events);
        self.audit.record(match &result {
            Ok(GenerationTerminal::Completed { token_count }) => AuditEvent::GenerationCompleted {
                model: request.model.clone(),
                token_count: *token_count,
            },
            Ok(GenerationTerminal::Cancelled) | Err(_) => AuditEvent::GenerationFailed {
                model: request.model.clone(),
            },
        })?;
        result
    }

    fn generate_inner(
        &self,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<GenerationTerminal> {
        if cancellation.is_cancelled() {
            return Ok(GenerationTerminal::Cancelled);
        }
        ensure_deadline(request)?;
        let manifest = self.registry.resolve(&request.model)?;
        if cancellation.is_cancelled() {
            return Ok(GenerationTerminal::Cancelled);
        }
        ensure_deadline(request)?;
        let model = self.artifacts.acquire(&manifest)?;
        if cancellation.is_cancelled() {
            return Ok(GenerationTerminal::Cancelled);
        }
        ensure_deadline(request)?;
        let mut tokens = TokenEventForwarder::new(events);
        match manifest.schema_version {
            MANIFEST_SCHEMA_VERSION if manifest.supports_verified_local_inference() => self
                .local
                .generate(&model, request, cancellation, &mut tokens),
            LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION
                if manifest.execution_plan.is_some()
                    && manifest.runtime_profile.as_deref() == Some(LOOM_RUNTIME_PROFILE) =>
            {
                self.sharded
                    .as_ref()
                    .ok_or(DomainError::BackendUnavailable)?
                    .generate(&model, request, cancellation, &mut tokens)
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
