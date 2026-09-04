use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use synapseflow_node::{
    AdmissionSettings, AuditSettings, KeycloakSettings, ListenerSettings, ModelPolicySettings,
    NodeProfile, NodeSettings, ShutdownSettings, StateSettings, TelemetrySettings,
};

pub(super) fn safe_defaults() -> NodeSettings {
    NodeSettings {
        profile: NodeProfile::Development,
        public_listener: ListenerSettings {
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 8_080)),
        },
        management_listener: ListenerSettings {
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 9_090)),
        },
        public_tls: None,
        trusted_proxy_addresses: BTreeSet::new(),
        keycloak: KeycloakSettings {
            issuer: "https://identity.invalid/realms/synapseflow".to_owned(),
            audience: "synapseflow-node".to_owned(),
            allowed_algorithms: BTreeSet::from(["RS256".to_owned()]),
            jwks_max_staleness_seconds: 3_600,
            clock_skew_seconds: 60,
        },
        admission: AdmissionSettings {
            max_request_bytes: 16 * 1024,
            max_concurrent_sessions: 1,
            max_sessions_per_principal: 1,
            max_queue_depth: 0,
        },
        state: StateSettings {
            database_path: PathBuf::from("synapseflow-state.db"),
        },
        model_policy: ModelPolicySettings {
            allowed_models: BTreeSet::new(),
        },
        audit: AuditSettings {
            directory: PathBuf::from("synapseflow-audit"),
            max_file_bytes: 10 * 1024 * 1024,
            max_file_age_seconds: 86_400,
            max_retained_files: 10,
        },
        telemetry: TelemetrySettings {
            queue_capacity: 256,
        },
        shutdown: ShutdownSettings { drain_seconds: 30 },
    }
}
