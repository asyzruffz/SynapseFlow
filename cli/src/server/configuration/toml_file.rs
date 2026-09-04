use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use synapseflow_node::NodeSettings;

use crate::error::CliError;

use super::{
    parser::{addresses, models, profile, socket_address},
    tls,
};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TomlSettings {
    profile: Option<String>,
    public: Option<PublicSettings>,
    management: Option<ListenerSettings>,
    keycloak: Option<KeycloakSettings>,
    admission: Option<AdmissionSettings>,
    state: Option<StateSettings>,
    audit: Option<AuditSettings>,
    telemetry: Option<TelemetrySettings>,
    model_policy: Option<ModelPolicySettings>,
    shutdown: Option<ShutdownSettings>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicSettings {
    bind: Option<String>,
    tls_cert_file: Option<PathBuf>,
    tls_key_file: Option<PathBuf>,
    trusted_proxy_addresses: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListenerSettings {
    bind: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeycloakSettings {
    issuer: Option<String>,
    audience: Option<String>,
    allowed_algorithms: Option<Vec<String>>,
    jwks_max_staleness_seconds: Option<u64>,
    clock_skew_seconds: Option<u64>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdmissionSettings {
    max_request_bytes: Option<usize>,
    max_concurrent_sessions: Option<usize>,
    max_sessions_per_principal: Option<usize>,
    max_queue_depth: Option<usize>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateSettings {
    database_path: Option<PathBuf>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditSettings {
    directory: Option<PathBuf>,
    max_file_bytes: Option<u64>,
    max_file_age_seconds: Option<u64>,
    max_retained_files: Option<usize>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TelemetrySettings {
    queue_capacity: Option<usize>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelPolicySettings {
    allowed_models: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShutdownSettings {
    drain_seconds: Option<u64>,
}

pub(super) fn read(path: Option<&Path>) -> Result<TomlSettings, CliError> {
    let path = match path {
        Some(path) => path.to_owned(),
        None => match env::var_os("SYNAPSEFLOW_CONFIG") {
            Some(path) => PathBuf::from(path),
            None => return Ok(TomlSettings::default()),
        },
    };
    let document = fs::read_to_string(path).map_err(|_| CliError::NodeConfigurationUnavailable)?;
    toml::from_str(&document).map_err(|_| CliError::NodeConfigurationInvalid)
}

pub(super) fn apply(settings: &mut NodeSettings, file: TomlSettings) -> Result<(), CliError> {
    if let Some(value) = file.profile {
        settings.profile = profile(&value)?;
    }
    if let Some(public) = file.public {
        if let Some(value) = public.bind {
            settings.public_listener.bind = socket_address(&value)?;
        }
        tls::apply(settings, public.tls_cert_file, public.tls_key_file)?;
        if let Some(value) = public.trusted_proxy_addresses {
            settings.trusted_proxy_addresses = addresses(value)?;
        }
    }
    if let Some(value) = file.management.and_then(|settings| settings.bind) {
        settings.management_listener.bind = socket_address(&value)?;
    }
    if let Some(keycloak) = file.keycloak {
        if let Some(value) = keycloak.issuer {
            settings.keycloak.issuer = value;
        }
        if let Some(value) = keycloak.audience {
            settings.keycloak.audience = value;
        }
        if let Some(value) = keycloak.allowed_algorithms {
            settings.keycloak.allowed_algorithms = value.into_iter().collect();
        }
        if let Some(value) = keycloak.jwks_max_staleness_seconds {
            settings.keycloak.jwks_max_staleness_seconds = value;
        }
        if let Some(value) = keycloak.clock_skew_seconds {
            settings.keycloak.clock_skew_seconds = value;
        }
    }
    if let Some(admission) = file.admission {
        if let Some(value) = admission.max_request_bytes {
            settings.admission.max_request_bytes = value;
        }
        if let Some(value) = admission.max_concurrent_sessions {
            settings.admission.max_concurrent_sessions = value;
        }
        if let Some(value) = admission.max_sessions_per_principal {
            settings.admission.max_sessions_per_principal = value;
        }
        if let Some(value) = admission.max_queue_depth {
            settings.admission.max_queue_depth = value;
        }
    }
    if let Some(value) = file.state.and_then(|settings| settings.database_path) {
        settings.state.database_path = value;
    }
    if let Some(audit) = file.audit {
        if let Some(value) = audit.directory {
            settings.audit.directory = value;
        }
        if let Some(value) = audit.max_file_bytes {
            settings.audit.max_file_bytes = value;
        }
        if let Some(value) = audit.max_file_age_seconds {
            settings.audit.max_file_age_seconds = value;
        }
        if let Some(value) = audit.max_retained_files {
            settings.audit.max_retained_files = value;
        }
    }
    if let Some(value) = file.telemetry.and_then(|settings| settings.queue_capacity) {
        settings.telemetry.queue_capacity = value;
    }
    if let Some(value) = file
        .model_policy
        .and_then(|settings| settings.allowed_models)
    {
        settings.model_policy.allowed_models = models(value)?;
    }
    if let Some(value) = file.shutdown.and_then(|settings| settings.drain_seconds) {
        settings.shutdown.drain_seconds = value;
    }
    Ok(())
}
