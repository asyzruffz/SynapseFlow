//! Application use cases composed exclusively from domain contracts and ports.

mod generation_orchestrator;
mod model_acquisition_service;
mod sharding;

#[cfg(test)]
mod tests;

pub use generation_orchestrator::GenerationOrchestrator;
pub use model_acquisition_service::ModelAcquisitionService;
pub use sharding::{
    ExecutionRoute, IdempotencyKey, RecoveryAttempt, SessionConfiguration, SessionManager,
    SessionSnapshot, ShardAssignment, ShardPlanner,
};
