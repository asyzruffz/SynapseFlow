use std::{env, path::PathBuf};

use synapseflow_node::NodeSettings;

use crate::error::CliError;

use super::{
    parser::{addresses, comma_list, models, number, profile, socket_address},
    tls,
};

pub(super) fn apply(settings: &mut NodeSettings) -> Result<(), CliError> {
    if let Some(value) = variable("PROFILE") {
        settings.profile = profile(&value)?;
    }
    if let Some(value) = variable("PUBLIC_BIND") {
        settings.public_listener.bind = socket_address(&value)?;
    }
    if let Some(value) = variable("MANAGEMENT_BIND") {
        settings.management_listener.bind = socket_address(&value)?;
    }
    tls::apply(
        settings,
        variable("PUBLIC_TLS_CERT_FILE").map(PathBuf::from),
        variable("PUBLIC_TLS_KEY_FILE").map(PathBuf::from),
    )?;
    if let Some(value) = variable("TRUSTED_PROXY_ADDRESSES") {
        settings.trusted_proxy_addresses = addresses(comma_list(value))?;
    }
    if let Some(value) = variable("KEYCLOAK_ISSUER") {
        settings.keycloak.issuer = value;
    }
    if let Some(value) = variable("KEYCLOAK_AUDIENCE") {
        settings.keycloak.audience = value;
    }
    if let Some(value) = variable("KEYCLOAK_ALLOWED_ALGORITHMS") {
        settings.keycloak.allowed_algorithms = comma_list(value).into_iter().collect();
    }
    if let Some(value) = variable("KEYCLOAK_JWKS_MAX_STALENESS_SECONDS") {
        settings.keycloak.jwks_max_staleness_seconds = number(&value)?;
    }
    if let Some(value) = variable("KEYCLOAK_CLOCK_SKEW_SECONDS") {
        settings.keycloak.clock_skew_seconds = number(&value)?;
    }
    if let Some(value) = variable("ADMISSION_MAX_REQUEST_BYTES") {
        settings.admission.max_request_bytes = number(&value)?;
    }
    if let Some(value) = variable("ADMISSION_MAX_CONCURRENT_SESSIONS") {
        settings.admission.max_concurrent_sessions = number(&value)?;
    }
    if let Some(value) = variable("ADMISSION_MAX_SESSIONS_PER_PRINCIPAL") {
        settings.admission.max_sessions_per_principal = number(&value)?;
    }
    if let Some(value) = variable("ADMISSION_MAX_QUEUE_DEPTH") {
        settings.admission.max_queue_depth = number(&value)?;
    }
    if let Some(value) = variable("AUDIT_DIRECTORY") {
        settings.audit.directory = PathBuf::from(value);
    }
    if let Some(value) = variable("AUDIT_MAX_FILE_BYTES") {
        settings.audit.max_file_bytes = number(&value)?;
    }
    if let Some(value) = variable("AUDIT_MAX_FILE_AGE_SECONDS") {
        settings.audit.max_file_age_seconds = number(&value)?;
    }
    if let Some(value) = variable("AUDIT_MAX_RETAINED_FILES") {
        settings.audit.max_retained_files = number(&value)?;
    }
    if let Some(value) = variable("TELEMETRY_QUEUE_CAPACITY") {
        settings.telemetry.queue_capacity = number(&value)?;
    }
    if let Some(value) = variable("MODEL_POLICY_ALLOWED_MODELS") {
        settings.model_policy.allowed_models = models(comma_list(value))?;
    }
    if let Some(value) = variable("SHUTDOWN_DRAIN_SECONDS") {
        settings.shutdown.drain_seconds = number(&value)?;
    }
    Ok(())
}

fn variable(name: &str) -> Option<String> {
    env::var(format!("SYNAPSEFLOW_{name}")).ok()
}
