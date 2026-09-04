use std::path::PathBuf;

use synapseflow_node::{NodeSettings, TlsSettings};

use crate::error::CliError;

/// Merges a single source's optional certificate/key pair without permitting
/// incomplete TLS material.
pub(super) fn apply(
    settings: &mut NodeSettings,
    certificate: Option<PathBuf>,
    key: Option<PathBuf>,
) -> Result<(), CliError> {
    let existing = settings.public_tls.clone();
    match (existing, certificate, key) {
        (None, None, None) => Ok(()),
        (Some(existing), certificate, key) => {
            settings.public_tls = Some(TlsSettings {
                certificate_path: certificate.unwrap_or(existing.certificate_path),
                private_key_path: key.unwrap_or(existing.private_key_path),
            });
            Ok(())
        }
        (None, Some(certificate_path), Some(private_key_path)) => {
            settings.public_tls = Some(TlsSettings {
                certificate_path,
                private_key_path,
            });
            Ok(())
        }
        (None, _, _) => Err(CliError::NodeConfigurationInvalid),
    }
}
