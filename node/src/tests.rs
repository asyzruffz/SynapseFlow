use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use super::{
    AdmissionSettings, AuditSettings, KeycloakSettings, ListenerSettings, ModelPolicySettings,
    NodeError, NodeProfile, NodeSettings, ShutdownSettings, StateSettings, TelemetrySettings,
    TlsSettings,
};

fn settings() -> NodeSettings {
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
            issuer: "https://identity.example/realms/synapseflow".to_owned(),
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
            database_path: PathBuf::from("state.db"),
        },
        model_policy: ModelPolicySettings {
            allowed_models: BTreeSet::new(),
        },
        audit: AuditSettings {
            directory: PathBuf::from("audit"),
            max_file_bytes: 1_024,
            max_file_age_seconds: 86_400,
            max_retained_files: 1,
        },
        telemetry: TelemetrySettings { queue_capacity: 64 },
        shutdown: ShutdownSettings { drain_seconds: 30 },
    }
}

#[test]
fn accepts_a_bounded_loopback_configuration() {
    assert_eq!(settings().validate(), Ok(()));
}

#[test]
fn rejects_an_exposed_public_listener_without_transport_protection() {
    let mut candidate = settings();
    candidate.profile = NodeProfile::Operational;
    candidate.public_listener.bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8_080));
    assert_eq!(
        candidate.validate(),
        Err(NodeError::PublicTransportUnprotected)
    );
}

#[test]
fn accepts_an_exposed_listener_with_complete_tls_material() {
    let mut candidate = settings();
    candidate.profile = NodeProfile::Operational;
    candidate.public_listener.bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8_080));
    candidate.public_tls = Some(TlsSettings {
        certificate_path: PathBuf::from("certificate.pem"),
        private_key_path: PathBuf::from("private-key.pem"),
    });
    assert_eq!(candidate.validate(), Ok(()));
}

#[test]
fn rejects_a_public_management_listener_and_invalid_keycloak_profile() {
    let mut candidate = settings();
    candidate.management_listener.bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 9_090));
    assert_eq!(
        candidate.validate(),
        Err(NodeError::ManagementListenerExposed)
    );

    let mut candidate = settings();
    candidate.keycloak.allowed_algorithms = BTreeSet::from(["none".to_owned()]);
    assert_eq!(
        candidate.validate(),
        Err(NodeError::KeycloakAlgorithmsInvalid)
    );
}

#[test]
fn accepts_a_configured_trusted_proxy_for_a_public_listener() {
    let mut candidate = settings();
    candidate.profile = NodeProfile::Operational;
    candidate.public_listener.bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8_080));
    candidate
        .trusted_proxy_addresses
        .insert(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(candidate.validate(), Ok(()));
}

#[test]
fn refuses_to_expose_a_development_listener_or_accept_unbounded_operational_settings() {
    let mut candidate = settings();
    candidate.public_listener.bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8_080));
    assert_eq!(
        candidate.validate(),
        Err(NodeError::DevelopmentListenerExposed)
    );

    let mut candidate = settings();
    candidate.telemetry.queue_capacity = 0;
    assert_eq!(
        candidate.validate(),
        Err(NodeError::TelemetrySettingsInvalid)
    );
}

#[test]
fn node_owns_a_kernel_workflow_per_application_session() {
    let server = super::NodeServer::new(settings()).expect("settings should be valid");
    let session_id =
        synapseflow_domain::PublicSessionId::new("application-session-0001".to_owned())
            .expect("fixture session should be valid");

    server
        .open_workflow(session_id.clone())
        .expect("node should create the client workflow");
    assert!(server.open_workflow(session_id).is_err());
}
