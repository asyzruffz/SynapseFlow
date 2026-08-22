use std::sync::Arc;

use synapseflow_domain::{DomainResult, DomainError, GenerationOutput, GenerationRequest};
use synapseflow_ports::{ArtifactStore, AuditEvent, AuditSink, ModelBackend, ModelRegistry};

/// Executes the complete local generation use case through abstract ports.
pub struct GenerationService {
    registry: Arc<dyn ModelRegistry>,
    artifacts: Arc<dyn ArtifactStore>,
    backend: Arc<dyn ModelBackend>,
    audit: Arc<dyn AuditSink>,
}

impl GenerationService {
    pub fn new(
        registry: Arc<dyn ModelRegistry>,
        artifacts: Arc<dyn ArtifactStore>,
        backend: Arc<dyn ModelBackend>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            registry,
            artifacts,
            backend,
            audit,
        }
    }

    /// Validates and executes a request without knowing a registry, cache, or backend implementation.
    pub fn generate(&self, request: GenerationRequest) -> DomainResult<GenerationOutput> {
        ensure_deadline(&request)?;
        self.audit.record(AuditEvent::GenerationStarted {
            model: request.model.clone(),
        })?;

        let result = self.generate_inner(&request);
        self.record_completion(&request, &result)?;
        result
    }

    fn generate_inner(&self, request: &GenerationRequest) -> DomainResult<GenerationOutput> {
        ensure_deadline(request)?;
        let manifest = self.registry.resolve(&request.model)?;
        ensure_deadline(request)?;
        let model = self.artifacts.acquire(&manifest)?;
        ensure_deadline(request)?;
        self.backend.generate(&model, request)
    }

    fn record_completion(
        &self,
        request: &GenerationRequest,
        result: &DomainResult<GenerationOutput>,
    ) -> DomainResult<()> {
        let event = match result {
            Ok(output) => AuditEvent::GenerationCompleted {
                model: request.model.clone(),
                token_count: output.tokens.len(),
            },
            Err(_) => AuditEvent::GenerationFailed {
                model: request.model.clone(),
            },
        };
        self.audit.record(event)
    }
}

fn ensure_deadline(request: &GenerationRequest) -> DomainResult<()> {
    if request.deadline_expired() {
        return Err(DomainError::DeadlineExceeded);
    }
    Ok(())
}
