use crux_core::capability::Operation;
use synapseflow_domain::{DomainResult, ModelConfig, ModelReference};

/// Requests that a shell compose a generation service for one model reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializeGeneration {
    pub model: ModelReference,
    pub config: ModelConfig,
}

impl Operation for InitializeGeneration {
    type Output = DomainResult<()>;
}
