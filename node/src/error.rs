use std::fmt;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    PublicTransportUnprotected,
    TlsIncomplete,
    ManagementListenerExposed,
    ListenerCollision,
    KeycloakIssuerInvalid,
    KeycloakAudienceInvalid,
    KeycloakAlgorithmsInvalid,
    JwksStalenessInvalid,
    AdmissionBoundsInvalid,
    AuditSettingsInvalid,
    DevelopmentListenerExposed,
    TelemetrySettingsInvalid,
    TelemetryExportUnavailable,
    ShutdownSettingsInvalid,
    ServerUnavailable,
    KeycloakClockSkewInvalid,
    AuditStorageUnavailable,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::PublicTransportUnprotected => "SYN-NODE-101",
            Self::TlsIncomplete => "SYN-NODE-102",
            Self::ManagementListenerExposed => "SYN-NODE-103",
            Self::ListenerCollision => "SYN-NODE-104",
            Self::KeycloakIssuerInvalid => "SYN-NODE-105",
            Self::KeycloakAudienceInvalid => "SYN-NODE-106",
            Self::KeycloakAlgorithmsInvalid => "SYN-NODE-107",
            Self::JwksStalenessInvalid => "SYN-NODE-108",
            Self::AdmissionBoundsInvalid => "SYN-NODE-109",
            Self::AuditSettingsInvalid => "SYN-NODE-110",
            Self::DevelopmentListenerExposed => "SYN-NODE-111",
            Self::TelemetrySettingsInvalid => "SYN-NODE-112",
            Self::TelemetryExportUnavailable => "SYN-NODE-113",
            Self::ShutdownSettingsInvalid => "SYN-NODE-114",
            Self::ServerUnavailable => "SYN-NODE-115",
            Self::KeycloakClockSkewInvalid => "SYN-NODE-116",
            Self::AuditStorageUnavailable => "SYN-NODE-117",
        };
        formatter.write_str(value)
    }
}

/// Stable safe configuration failures for node startup.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum NodeError {
    #[error("a non-loopback public listener requires direct TLS or a trusted proxy")]
    PublicTransportUnprotected,
    #[error("configured TLS material is incomplete")]
    TlsIncomplete,
    #[error("the management listener must be loopback-only")]
    ManagementListenerExposed,
    #[error("public and management listeners must be distinct")]
    ListenerCollision,
    #[error("the Keycloak issuer must use HTTPS")]
    KeycloakIssuerInvalid,
    #[error("the Keycloak audience must be present")]
    KeycloakAudienceInvalid,
    #[error("at least one asymmetric Keycloak signing algorithm is required")]
    KeycloakAlgorithmsInvalid,
    #[error("the JWKS maximum staleness must be positive")]
    JwksStalenessInvalid,
    #[error("node admission bounds must be positive")]
    AdmissionBoundsInvalid,
    #[error("node audit storage settings are invalid")]
    AuditSettingsInvalid,
    #[error("development profile may bind the public listener only to loopback")]
    DevelopmentListenerExposed,
    #[error("node telemetry settings are invalid")]
    TelemetrySettingsInvalid,
    #[error("node telemetry exporter is unavailable")]
    TelemetryExportUnavailable,
    #[error("node shutdown settings are invalid")]
    ShutdownSettingsInvalid,
    #[error("node server listener could not start")]
    ServerUnavailable,
    #[error("the Keycloak clock-skew allowance is invalid")]
    KeycloakClockSkewInvalid,
    #[error("node audit storage is unavailable")]
    AuditStorageUnavailable,
}

impl NodeError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::PublicTransportUnprotected => ErrorCode::PublicTransportUnprotected,
            Self::TlsIncomplete => ErrorCode::TlsIncomplete,
            Self::ManagementListenerExposed => ErrorCode::ManagementListenerExposed,
            Self::ListenerCollision => ErrorCode::ListenerCollision,
            Self::KeycloakIssuerInvalid => ErrorCode::KeycloakIssuerInvalid,
            Self::KeycloakAudienceInvalid => ErrorCode::KeycloakAudienceInvalid,
            Self::KeycloakAlgorithmsInvalid => ErrorCode::KeycloakAlgorithmsInvalid,
            Self::JwksStalenessInvalid => ErrorCode::JwksStalenessInvalid,
            Self::AdmissionBoundsInvalid => ErrorCode::AdmissionBoundsInvalid,
            Self::AuditSettingsInvalid => ErrorCode::AuditSettingsInvalid,
            Self::DevelopmentListenerExposed => ErrorCode::DevelopmentListenerExposed,
            Self::TelemetrySettingsInvalid => ErrorCode::TelemetrySettingsInvalid,
            Self::TelemetryExportUnavailable => ErrorCode::TelemetryExportUnavailable,
            Self::ShutdownSettingsInvalid => ErrorCode::ShutdownSettingsInvalid,
            Self::ServerUnavailable => ErrorCode::ServerUnavailable,
            Self::KeycloakClockSkewInvalid => ErrorCode::KeycloakClockSkewInvalid,
            Self::AuditStorageUnavailable => ErrorCode::AuditStorageUnavailable,
        }
    }
}
