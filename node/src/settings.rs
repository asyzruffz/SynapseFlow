use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use crate::NodeError;
use synapseflow_domain::ModelReference;

/// Explicit operating profile. Development is unable to expose a public listener.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeProfile {
    Development,
    Operational,
}

/// Validated listener settings for one node endpoint class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListenerSettings {
    pub bind: SocketAddr,
}

/// File-backed TLS material for a directly terminated public listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TlsSettings {
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
}

impl TlsSettings {
    fn is_complete(&self) -> bool {
        !self.certificate_path.as_os_str().is_empty()
            && !self.private_key_path.as_os_str().is_empty()
    }
}

/// Keycloak access-token validation inputs owned by the node adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeycloakSettings {
    pub issuer: String,
    pub audience: String,
    pub allowed_algorithms: BTreeSet<String>,
    pub jwks_max_staleness_seconds: u64,
    pub clock_skew_seconds: u64,
}

/// Bounds applied before a request reaches model acquisition or execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionSettings {
    pub max_request_bytes: usize,
    pub max_prompt_bytes: usize,
    pub max_output_tokens: u16,
    pub max_deadline_ms: u64,
    pub max_concurrent_sessions: usize,
    pub max_sessions_per_principal: usize,
    pub max_queue_depth: usize,
}

/// Durable single-node control-plane database location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSettings {
    pub database_path: PathBuf,
}

/// Immutable models that a caller with `synapseflow:generate` may execute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelPolicySettings {
    pub allowed_models: BTreeSet<ModelReference>,
}

/// Durable node-local audit storage bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSettings {
    pub directory: PathBuf,
    pub max_file_bytes: u64,
    pub max_file_age_seconds: u64,
    pub max_retained_files: usize,
}

/// Bounded, non-authoritative telemetry delivery settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetrySettings {
    pub queue_capacity: usize,
}

/// Process-drain limit owned by the CLI server process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownSettings {
    pub drain_seconds: u64,
}

/// Complete framework-free settings required before the node can listen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeSettings {
    pub profile: NodeProfile,
    pub public_listener: ListenerSettings,
    pub management_listener: ListenerSettings,
    pub public_tls: Option<TlsSettings>,
    pub trusted_proxy_addresses: BTreeSet<IpAddr>,
    pub keycloak: KeycloakSettings,
    pub admission: AdmissionSettings,
    pub state: StateSettings,
    pub model_policy: ModelPolicySettings,
    pub audit: AuditSettings,
    pub telemetry: TelemetrySettings,
    pub shutdown: ShutdownSettings,
}

impl NodeSettings {
    /// Rejects configurations that would expose an unauthenticated or unsafe node boundary.
    pub fn validate(&self) -> Result<(), NodeError> {
        validate_profile(self)?;
        validate_public_listener(self)?;
        validate_management_listener(self)?;
        validate_keycloak(&self.keycloak)?;
        validate_admission(&self.admission)?;
        validate_state(&self.state)?;
        validate_audit(&self.audit)?;
        validate_telemetry(&self.telemetry)?;
        validate_shutdown(&self.shutdown)
    }
}

fn validate_state(settings: &StateSettings) -> Result<(), NodeError> {
    (!settings.database_path.as_os_str().is_empty())
        .then_some(())
        .ok_or(NodeError::StateSettingsInvalid)
}

fn validate_profile(settings: &NodeSettings) -> Result<(), NodeError> {
    if settings.profile == NodeProfile::Development
        && (!settings.public_listener.bind.ip().is_loopback()
            || !settings.trusted_proxy_addresses.is_empty())
    {
        return Err(NodeError::DevelopmentListenerExposed);
    }
    Ok(())
}

fn validate_public_listener(settings: &NodeSettings) -> Result<(), NodeError> {
    match &settings.public_tls {
        Some(tls) if !tls.is_complete() => Err(NodeError::TlsIncomplete),
        Some(_) => Ok(()),
        None if settings.public_listener.bind.ip().is_loopback()
            || !settings.trusted_proxy_addresses.is_empty() =>
        {
            Ok(())
        }
        None => Err(NodeError::PublicTransportUnprotected),
    }
}

fn validate_management_listener(settings: &NodeSettings) -> Result<(), NodeError> {
    if settings.public_listener.bind == settings.management_listener.bind {
        return Err(NodeError::ListenerCollision);
    }
    if !settings.management_listener.bind.ip().is_loopback() {
        return Err(NodeError::ManagementListenerExposed);
    }
    Ok(())
}

fn validate_keycloak(settings: &KeycloakSettings) -> Result<(), NodeError> {
    if !settings.issuer.starts_with("https://") {
        return Err(NodeError::KeycloakIssuerInvalid);
    }
    if settings.audience.trim().is_empty() {
        return Err(NodeError::KeycloakAudienceInvalid);
    }
    if settings.allowed_algorithms.is_empty()
        || settings.allowed_algorithms.iter().any(|algorithm| {
            !matches!(
                algorithm.as_str(),
                "RS256" | "RS384" | "RS512" | "ES256" | "ES384" | "ES512" | "EdDSA"
            )
        })
    {
        return Err(NodeError::KeycloakAlgorithmsInvalid);
    }
    if settings.jwks_max_staleness_seconds == 0 {
        return Err(NodeError::JwksStalenessInvalid);
    }
    if settings.clock_skew_seconds > 300 {
        return Err(NodeError::KeycloakClockSkewInvalid);
    }
    Ok(())
}

fn validate_admission(settings: &AdmissionSettings) -> Result<(), NodeError> {
    if settings.max_request_bytes == 0
        || settings.max_prompt_bytes == 0
        || settings.max_output_tokens == 0
        || settings.max_output_tokens > 256
        || settings.max_deadline_ms == 0
        || settings.max_concurrent_sessions == 0
        || settings.max_sessions_per_principal == 0
    {
        return Err(NodeError::AdmissionBoundsInvalid);
    }
    Ok(())
}

fn validate_audit(settings: &AuditSettings) -> Result<(), NodeError> {
    if settings.directory.as_os_str().is_empty()
        || settings.max_file_bytes == 0
        || settings.max_file_age_seconds == 0
        || settings.max_retained_files == 0
    {
        return Err(NodeError::AuditSettingsInvalid);
    }
    Ok(())
}

fn validate_telemetry(settings: &TelemetrySettings) -> Result<(), NodeError> {
    (settings.queue_capacity > 0)
        .then_some(())
        .ok_or(NodeError::TelemetrySettingsInvalid)
}

fn validate_shutdown(settings: &ShutdownSettings) -> Result<(), NodeError> {
    (settings.drain_seconds > 0)
        .then_some(())
        .ok_or(NodeError::ShutdownSettingsInvalid)
}
