use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::request::ProvisioningRequest;

/// Command line for explicit fixture-manifest provisioning.
#[derive(Parser)]
#[command(name = "synapseflow-fixture-provisioner")]
#[command(about = "Create a canonical, signed SynapseFlow fixture manifest")]
pub(crate) struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Hash one local GGUF and write its signed immutable manifest.
    Manifest(ManifestArgs),
}

#[derive(Args)]
struct ManifestArgs {
    /// Local GGUF fixture. This path is read but never copied into the repository.
    #[arg(long)]
    artifact: PathBuf,

    /// Approved HTTPS URI recorded in the manifest for this artifact.
    #[arg(long)]
    artifact_uri: String,

    /// File containing an unpadded base64url Ed25519 32-byte signing seed.
    #[arg(long)]
    signing_key: PathBuf,

    /// New output path for the canonical signed manifest. Existing files are refused.
    #[arg(long)]
    output: PathBuf,

    #[arg(long, default_value = "tinyllama-chat")]
    model_id: String,

    #[arg(long, default_value = "1.1b-q5km-2026-08-22")]
    model_version: String,

    #[arg(long, default_value = "Apache-2.0")]
    license: String,

    #[arg(long, default_value = "fixture:tinyllama")]
    provenance: String,
}

impl CommandLine {
    pub(crate) fn into_request(self) -> ProvisioningRequest {
        match self.command {
            Command::Manifest(arguments) => ProvisioningRequest {
                artifact_path: arguments.artifact,
                artifact_uri: arguments.artifact_uri,
                signing_key_path: arguments.signing_key,
                output_path: arguments.output,
                model_id: arguments.model_id,
                model_version: arguments.model_version,
                publisher_key_id: "ed25519:synapseflow-fixture-2026-08".to_owned(),
                license: arguments.license,
                provenance: arguments.provenance,
            },
        }
    }
}
