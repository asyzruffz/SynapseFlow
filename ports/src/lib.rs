//! Object-safe boundaries between SynapseFlow application logic and adapters.

mod admission;
mod artifact_store;
mod audit;
mod backend;
mod clock;
mod identity;
mod model_policy;
mod peer_directory;
mod registry;
mod session_store;
mod shard_execution;
mod sharded_generation;
mod telemetry;
mod tokenizer;
mod transport;
mod worker;

pub use admission::{AdmissionAccounting, AdmissionRequest};
pub use artifact_store::{
    ArtifactStore, CacheEntryState, CachedArtifactInspection, ModelCacheInspection, VerifiedModel,
};
pub use audit::{AuditEvent, AuditSink, NodeSessionAudit, ShardSessionOutcome};
pub use backend::ModelBackend;
pub use clock::Clock;
pub use identity::{BearerCredential, IdentityVerifier};
pub use model_policy::ModelAccessPolicy;
pub use peer_directory::PeerDirectory;
pub use registry::ModelRegistry;
pub use session_store::{
    ActiveSessionControl, CreateSessionResult, DurableSession, RequestFingerprint, SessionStore,
};
pub use shard_execution::{
    ExecutionCancellation, NeverCancelled, ShardExecutionBackend, ShardExecutionOutput,
    ShardExecutionRequest, ShardExecutionRequirements,
};
pub use sharded_generation::ShardedGenerationRuntime;
pub use telemetry::{TelemetryEvent, TelemetrySink};
pub use tokenizer::ModelTokenizer;
pub use transport::{ReceivedFrame, Transport, TransportReceipt};
pub use worker::{ShardAvailability, WorkerCapability, WorkerHealth, WorkerId};
