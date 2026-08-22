//! Creates canonical, signed manifests for explicitly provisioned local fixtures.

mod args;
mod artifact;
mod manifest;
mod output;
mod request;
#[cfg(test)]
mod tests;

use std::process::ExitCode;

use clap::Parser;

use crate::{args::CommandLine, artifact::ArtifactFingerprint, manifest::SignedManifest};

fn main() -> ExitCode {
    let command = CommandLine::parse();
    match run(command) {
        Ok(receipt) => {
            println!("manifest: {}", receipt.manifest_path.display());
            println!("reference: {}", receipt.reference);
            println!("publisher public key: {}", receipt.public_key_base64url);
            println!("artifact size: {}", receipt.artifact_size_bytes);
            println!("artifact sha256: {}", receipt.artifact_sha256);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("fixture provisioning failed: {message}");
            ExitCode::FAILURE
        }
    }
}

struct ProvisioningReceipt {
    manifest_path: std::path::PathBuf,
    reference: String,
    public_key_base64url: String,
    artifact_size_bytes: u64,
    artifact_sha256: String,
}

fn run(command: CommandLine) -> Result<ProvisioningReceipt, String> {
    let request = command.into_request();
    let artifact = ArtifactFingerprint::from_file(&request.artifact_path)?;
    let signed_manifest = SignedManifest::create(&request, &artifact)?;
    output::write_new(&request.output_path, &signed_manifest.document)?;

    Ok(ProvisioningReceipt {
        manifest_path: request.output_path,
        reference: signed_manifest.reference,
        public_key_base64url: signed_manifest.public_key_base64url,
        artifact_size_bytes: artifact.size_bytes,
        artifact_sha256: artifact.content_sha256,
    })
}
