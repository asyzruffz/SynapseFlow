use std::path::PathBuf;

use clap::Args;
use synapseflow_domain::{
    DomainResult, GenerationPolicy, GenerationRequest, ModelConfig, ModelReference,
};

use super::args::VerifiedLocalRuntimeArgs;

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
    /// Write output to a new, explicit file. Existing files are never overwritten.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Emit the session ID, decoded text, and token IDs as JSON.
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    runtime: VerifiedLocalRuntimeArgs,
}

impl RunCommand {
    /// Converts parsed user input into the framework-independent request contract.
    pub fn into_parts(
        self,
    ) -> DomainResult<(GenerationRequest, Option<PathBuf>, bool, ModelConfig)> {
        let reference = ModelReference::parse(self.model)?;
        let policy =
            GenerationPolicy::new(self.max_tokens, self.temperature, self.top_p, self.seed)?;
        let request = GenerationRequest::new(reference, self.prompt, policy)?;
        let runtime = self.runtime.into_config()?;
        Ok((request, self.output, self.json, runtime))
    }
}
