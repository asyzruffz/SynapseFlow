use std::path::Path;

use synapseflow_domain::execution::{LayerRange, RemainingDeadline, SessionId};
use synapseflow_domain::{DomainResult, ModelReference};
use synapseflow_ports::ExecutionCancellation;

use crate::input::StageInput;

/// Loom's inspected model dimensions for a verified artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoomModelLayout {
    pub layer_count: u32,
    pub activation_width: u32,
    pub vocabulary_size: u32,
}

/// A validated request passed from the adapter to Loom's execution engine.
#[derive(Clone, Debug, PartialEq)]
pub struct LoomExecutionRequest {
    pub model: ModelReference,
    pub session_id: SessionId,
    pub declared_range: LayerRange,
    pub input: StageInput,
    pub final_stage: bool,
    pub remaining_deadline: RemainingDeadline,
}

/// A result returned by Loom's execution engine.
#[derive(Clone, Debug, PartialEq)]
pub enum LoomExecutionOutput {
    Boundary {
        activations: Vec<f32>,
        token_count: usize,
    },
    FinalLogits(Vec<f32>),
}

/// Seam between the strategy adapter and the Llama-specific Loom engine.
///
/// It permits generated-fixture tests without exposing runtime types outside
/// this adapter crate. The production implementation owns GGUF inspection, KV
/// state, and cancellation polling.
pub trait LoomExecutor: Send + Sync {
    fn inspect(&self, artifact: &Path) -> DomainResult<LoomModelLayout>;

    fn execute(
        &self,
        artifact: &Path,
        request: &LoomExecutionRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<LoomExecutionOutput>;

    fn release_session(&self, _: &Path, _: LayerRange, _: &SessionId) -> DomainResult<()> {
        Ok(())
    }
}
