//! Configuration assembly for the `synapseflow serve` command.
//!
//! Settings flow from safe defaults through TOML, environment, and explicit
//! command-line overrides. Each source is deliberately isolated so precedence
//! remains evident and independently testable.

mod command_line;
mod defaults;
mod environment;
mod parser;
mod tls;
mod toml_file;

use std::path::PathBuf;

use synapseflow_domain::{ModelConfig, ModelReference};
use synapseflow_node::NodeSettings;

use crate::{commands::ServeCommand, error::CliError};

pub(super) struct ServeSettings {
    pub model: ModelReference,
    pub config: ModelConfig,
    pub node: NodeSettings,
}

#[derive(Default)]
pub(super) struct RuntimeOverrides {
    model: Option<String>,
    manifest_path: Option<PathBuf>,
    artifact_path: Option<PathBuf>,
    cache_directory: Option<PathBuf>,
    publisher_public_key: Option<String>,
}

pub(super) fn load_settings(command: &ServeCommand) -> Result<ServeSettings, CliError> {
    let mut settings = defaults::safe_defaults();
    let mut runtime = RuntimeOverrides::default();
    let file = toml_file::read(command.config.as_deref())?;
    toml_file::apply(&mut settings, &mut runtime, file)?;
    environment::apply(&mut settings, &mut runtime)?;
    command_line::apply(&mut settings, &mut runtime, command)?;
    settings
        .validate()
        .map_err(|_| CliError::NodeConfigurationInvalid)?;
    let (model, config) = runtime.resolve()?;
    Ok(ServeSettings {
        model,
        config,
        node: settings,
    })
}

impl RuntimeOverrides {
    fn resolve(self) -> Result<(ModelReference, ModelConfig), CliError> {
        let model = ModelReference::parse(self.model.ok_or(CliError::NodeConfigurationInvalid)?)
            .map_err(|_| CliError::NodeConfigurationInvalid)?;
        let config = ModelConfig {
            manifest_path: self
                .manifest_path
                .ok_or(CliError::NodeConfigurationInvalid)?,
            artifact_path: self
                .artifact_path
                .ok_or(CliError::NodeConfigurationInvalid)?,
            cache_directory: self
                .cache_directory
                .ok_or(CliError::NodeConfigurationInvalid)?,
            publisher_public_key: self
                .publisher_public_key
                .ok_or(CliError::NodeConfigurationInvalid)?,
        };
        config
            .validate()
            .map_err(|_| CliError::NodeConfigurationInvalid)?;
        Ok((model, config))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        defaults::safe_defaults,
        toml_file::{apply, TomlSettings},
    };

    #[test]
    fn rejects_unknown_configuration_keys() {
        let file = toml::from_str::<TomlSettings>("unexpected = true");
        assert!(file.is_err());
    }

    #[test]
    fn file_configuration_changes_only_declared_values() {
        let file = toml::from_str::<TomlSettings>(
            "[admission]\nmax_request_bytes = 4096\nmax_prompt_bytes = 2048\nmax_output_tokens = 64\nmax_deadline_ms = 15000",
        )
        .expect("fixture TOML");
        let mut settings = safe_defaults();
        let mut runtime = super::RuntimeOverrides::default();
        apply(&mut settings, &mut runtime, file).expect("configuration should apply");
        assert_eq!(settings.admission.max_request_bytes, 4096);
        assert_eq!(settings.admission.max_prompt_bytes, 2048);
        assert_eq!(settings.admission.max_output_tokens, 64);
        assert_eq!(settings.admission.max_deadline_ms, 15_000);
        assert_eq!(settings.admission.max_concurrent_sessions, 1);
    }
}
