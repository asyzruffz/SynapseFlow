//! SynapseFlow Inference Library - minimal stub for MVP
//!
//! Exposes a lightweight API the CLI can call. The heavy-lifting (candle integration)
//! is feature-gated and will be added behind the `with-candle` feature.

pub mod backends;
pub mod config;
pub mod error;

use synapseflow_core::models::source::ModelSource;

use backends::candle::llama::LlamaModel;
use backends::BackendType;
use config::InferenceConfig;
use error::{InferenceError, Result};

pub trait InferenceEngine {
    fn initialize(source: ModelSource) -> Result<Self>
    where
        Self: Sized;

    fn generate(
        &self,
        prompt: &str,
        config: InferenceConfig,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<()>;
}

pub fn load_model(source: ModelSource, backend: BackendType) -> Result<Box<dyn InferenceEngine>> {
    match backend {
        BackendType::Candle => {
            let model = LlamaModel::initialize(source)?;
            Ok(Box::new(model))
        }
        BackendType::LlamaCpp => Err(InferenceError::BackendUnavailable { backend }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{backends::BackendType, error::InferenceError, load_model};
    use synapseflow_core::models::source::ModelSource;

    #[test]
    fn unsupported_backend_fails_before_model_access() {
        let source = ModelSource::from(PathBuf::from("model-is-not-needed-for-this-test"));

        match load_model(source, BackendType::LlamaCpp) {
            Err(InferenceError::BackendUnavailable {
                backend: BackendType::LlamaCpp,
            }) => {}
            Err(error) => panic!("expected backend-unavailable error, got {error}"),
            Ok(_) => panic!("an unsupported backend must not create an inference engine"),
        }
    }
}
