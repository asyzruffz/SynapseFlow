use std::path::PathBuf;

use clap::Args;
use synapseflow_domain::{DomainResult, GenerationPolicy, GenerationRequest, ModelReference};

use crate::runtime::VerifiedLocalRuntimeConfig;

/// CLI parsing for the explicit verified-local runtime configuration.
#[derive(Args)]
pub(super) struct VerifiedLocalRuntimeArgs {
    /// Provisioned, signed manifest document for the selected immutable reference.
    #[arg(long)]
    manifest: PathBuf,
    /// Provisioned GGUF source matching the signed manifest artifact declaration.
    #[arg(long)]
    artifact: PathBuf,
    /// Directory for the verified content-addressed local cache.
    #[arg(long)]
    cache_dir: PathBuf,
    /// Base64url Ed25519 public key for the configured fixture publisher.
    #[arg(long)]
    publisher_public_key: String,
}

impl VerifiedLocalRuntimeArgs {
    fn into_config(self) -> DomainResult<VerifiedLocalRuntimeConfig> {
        let config = VerifiedLocalRuntimeConfig {
            manifest_path: self.manifest,
            artifact_path: self.artifact,
            cache_directory: self.cache_dir,
            publisher_public_key: self.publisher_public_key,
        };
        config.validate()?;
        Ok(config)
    }
}

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
    ) -> DomainResult<(
        GenerationRequest,
        Option<PathBuf>,
        bool,
        VerifiedLocalRuntimeConfig,
    )> {
        let reference = ModelReference::parse(self.model)?;
        let policy =
            GenerationPolicy::new(self.max_tokens, self.temperature, self.top_p, self.seed)?;
        let request = GenerationRequest::new(reference, self.prompt, policy)?;
        let runtime = self.runtime.into_config()?;
        Ok((request, self.output, self.json, runtime))
    }
}
