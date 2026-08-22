use clap::Args;
use synapseflow_domain::{DomainResult, GenerationPolicy, GenerationRequest, ModelReference};

/// Arguments for one local generation request.
#[derive(Args)]
pub struct RunCommand {
    /// Versioned manifest reference, never a raw model path or URL.
    #[arg(long)]
    model: String,
    #[arg(long)]
    prompt: String,
    #[arg(long, default_value_t = 16)]
    max_tokens: u16,
    #[arg(long, default_value_t = 0.7)]
    temperature: f32,
    #[arg(long, default_value_t = 0.9)]
    top_p: f32,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

impl RunCommand {
    /// Converts parsed user input into the framework-independent request contract.
    pub fn into_request(self) -> DomainResult<GenerationRequest> {
        let reference = ModelReference::parse(self.model)?;
        let policy =
            GenerationPolicy::new(self.max_tokens, self.temperature, self.top_p, self.seed)?;
        GenerationRequest::new(reference, self.prompt, policy)
    }
}
