use synapseflow_domain::{DomainResult, GenerationOutput, GenerationRequest};

use crate::VerifiedModel;

/// Executes a verified sharded model after the application selects its profile.
///
/// Implementations own tokenization and stage-local runtime details. They must
/// not select a manifest, route, retry policy, or public generation policy.
pub trait ShardedGenerationRuntime: Send + Sync {
    fn generate(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
    ) -> DomainResult<GenerationOutput>;
}
