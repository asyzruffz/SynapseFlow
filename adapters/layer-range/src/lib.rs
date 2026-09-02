//! Loom's Llama-specific `layer_range_v1` adapter behind a strategy-neutral port.

mod backend;
mod compatibility;
mod input;
mod loom;
mod runtime;
mod tokenizer;

pub use backend::LoomBackend;
pub use input::StageInput;
pub use loom::LoomEngine;
pub use runtime::{LoomExecutionOutput, LoomExecutionRequest, LoomExecutor, LoomModelLayout};
pub use tokenizer::LoomTokenizer;

#[cfg(test)]
mod tests;
