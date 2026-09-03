//! Application use cases composed exclusively from domain contracts and ports.

mod generation_orchestrator;
mod live_generation;
mod model_acquisition_service;
mod session_execution_service;
mod session_manager;
mod sharding;

#[cfg(test)]
mod tests;

pub use generation_orchestrator::GenerationOrchestrator;
pub use model_acquisition_service::ModelAcquisitionService;
pub use session_execution_service::SessionExecutionService;
pub use session_manager::{
    GenerationSessionManager, SessionStartRequest, SessionStartResult, SessionTerminal,
};
pub use sharding::{
    ExecutionRoute, IdempotencyKey, LayerRangeShardedGenerationRuntime, RecoveryAttempt,
    SessionConfiguration, SessionManager, SessionSnapshot, ShardAssignment, ShardPlanner,
};
