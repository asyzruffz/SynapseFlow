use synapseflow_domain::execution::SessionId;
use synapseflow_domain::{DomainResult, ModelReference, ShardId};

use crate::WorkerId;

/// Privacy-safe terminal outcome of a distributed shard session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShardSessionOutcome {
    Completed,
    Cancelled,
    Failed,
    Recovered,
}

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
    ManifestVerified {
        model: ModelReference,
        publisher_key_id: String,
    },
    ArtifactsCached {
        model: ModelReference,
        artifact_count: usize,
    },
    ModelAcquisitionFailed {
        model: ModelReference,
    },
    ShardSessionFinished {
        model: ModelReference,
        shard: ShardId,
        worker: WorkerId,
        session_id: SessionId,
        outcome: ShardSessionOutcome,
        retry_count: u8,
        fallback_count: u8,
    },
}

/// Emits privacy-safe lifecycle events.
pub trait AuditSink: Send + Sync {
    fn record(&self, event: AuditEvent) -> DomainResult<()>;
}
