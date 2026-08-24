//! Application use cases composed exclusively from domain contracts and ports.

mod generation_service;
mod model_acquisition_service;
mod sharding;

#[cfg(test)]
mod tests;

pub use generation_service::GenerationService;
pub use model_acquisition_service::ModelAcquisitionService;
pub use sharding::{
    ExecutionRoute, IdempotencyKey, RecoveryAttempt, SessionConfiguration, SessionManager,
    SessionSnapshot, ShardAssignment, ShardPlanner,
};
