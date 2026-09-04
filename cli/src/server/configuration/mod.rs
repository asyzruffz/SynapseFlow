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

use synapseflow_node::NodeSettings;

use crate::{commands::ServeCommand, error::CliError};

pub(super) fn load_settings(command: &ServeCommand) -> Result<NodeSettings, CliError> {
    let mut settings = defaults::safe_defaults();
    let file = toml_file::read(command.config.as_deref())?;
    toml_file::apply(&mut settings, file)?;
    environment::apply(&mut settings)?;
    command_line::apply(&mut settings, command)?;
    settings
        .validate()
        .map_err(|_| CliError::NodeConfigurationInvalid)?;
    Ok(settings)
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
        let file = toml::from_str::<TomlSettings>("[admission]\nmax_request_bytes = 4096")
            .expect("fixture TOML");
        let mut settings = safe_defaults();
        apply(&mut settings, file).expect("configuration should apply");
        assert_eq!(settings.admission.max_request_bytes, 4096);
        assert_eq!(settings.admission.max_concurrent_sessions, 1);
    }
}
