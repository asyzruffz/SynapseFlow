//! Llama-specific `layer_range_v1` execution adapter behind a strategy-neutral port.

mod backend;
mod compatibility;
mod input;
mod runtime;

pub use backend::LayerRangeBackend;
pub use input::StageInput;
pub use runtime::{
    LayerRangeRuntime, NativeExecutionOutput, NativeLayerRangeRequest, NativeModelLayout,
};

#[cfg(test)]
mod tests;
