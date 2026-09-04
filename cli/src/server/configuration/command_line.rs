use synapseflow_node::NodeSettings;

use crate::{commands::ServeCommand, error::CliError};

use super::{
    parser::{addresses, models, profile, socket_address},
    tls,
};

pub(super) fn apply(settings: &mut NodeSettings, command: &ServeCommand) -> Result<(), CliError> {
    if let Some(value) = &command.profile {
        settings.profile = profile(value)?;
    }
    if let Some(value) = &command.public_bind {
        settings.public_listener.bind = socket_address(value)?;
    }
    if let Some(value) = &command.management_bind {
        settings.management_listener.bind = socket_address(value)?;
    }
    tls::apply(
        settings,
        command.public_tls_cert_file.clone(),
        command.public_tls_key_file.clone(),
    )?;
    if !command.trusted_proxy_address.is_empty() {
        settings.trusted_proxy_addresses = addresses(command.trusted_proxy_address.clone())?;
    }
    if let Some(value) = &command.keycloak_issuer {
        settings.keycloak.issuer = value.clone();
    }
    if let Some(value) = &command.keycloak_audience {
        settings.keycloak.audience = value.clone();
    }
    if !command.keycloak_allowed_algorithm.is_empty() {
        settings.keycloak.allowed_algorithms =
            command.keycloak_allowed_algorithm.iter().cloned().collect();
    }
    if let Some(value) = command.keycloak_jwks_max_staleness_seconds {
        settings.keycloak.jwks_max_staleness_seconds = value;
    }
    if let Some(value) = command.keycloak_clock_skew_seconds {
        settings.keycloak.clock_skew_seconds = value;
    }
    if let Some(value) = &command.audit_directory {
        settings.audit.directory = value.clone();
    }
    if !command.allowed_model.is_empty() {
        settings.model_policy.allowed_models = models(command.allowed_model.clone())?;
    }
    Ok(())
}
