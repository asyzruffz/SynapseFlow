use std::path::Path;

use synapseflow_domain::execution::{LayerRange, RemainingDeadline};
use synapseflow_domain::{DomainResult, ModelReference};
use synapseflow_ports::ExecutionCancellation;

use crate::input::StageInput;

/// Adapter-owned description returned by the private native bridge after GGUF inspection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeModelLayout {
    pub layer_count: u32,
    pub activation_width: u32,
    pub vocabulary_size: u32,
}

/// Input passed across the adapter's private native boundary, never across the network.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeLayerRangeRequest {
    pub model: ModelReference,
    pub declared_range: LayerRange,
    pub input: StageInput,
    pub final_stage: bool,
    pub remaining_deadline: RemainingDeadline,
}

/// Result returned by the adapter's private native boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum NativeExecutionOutput {
    Boundary(Vec<f32>),
    FinalLogits(Vec<f32>),
}

/// Private-native-bridge seam for the Llama-specific adapter.
///
/// Implementations own their C ABI, native handles, GGUF inspection, KV state,
/// and cancellation polling. No native type crosses into domain, ports, or
/// application crates.
pub trait LayerRangeRuntime: Send + Sync {
    fn inspect(&self, artifact: &Path) -> DomainResult<NativeModelLayout>;

    fn execute(
        &self,
        artifact: &Path,
        request: &NativeLayerRangeRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<NativeExecutionOutput>;
}
