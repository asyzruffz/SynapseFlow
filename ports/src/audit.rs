use synapseflow_domain::{DomainResult, ModelReference};

/// A safe, payload-free event emitted around a generation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEvent {
    GenerationStarted {
        model: ModelReference,
    },
    GenerationCompleted {
        model: ModelReference,
        token_count: usize,
    },
    GenerationFailed {
        model: ModelReference,
    },
}

/// Emits privacy-safe lifecycle events.
pub trait AuditSink: Send + Sync {
    fn record(&self, event: AuditEvent) -> DomainResult<()>;
}
