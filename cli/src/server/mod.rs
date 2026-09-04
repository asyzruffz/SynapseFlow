mod configuration;
mod telemetry;

use configuration::{load_settings, ServeSettings};
use telemetry::StderrTelemetryExporter;

use std::{process::ExitCode, sync::Arc};

use synapseflow_adapter_sqlite_state::{SqliteNodeState, SqliteNodeStateSettings};
use synapseflow_application::{GenerationSessionManager, SessionExecutionService};
use synapseflow_node::{
    BoundedTelemetrySink, ConfiguredModelAccessPolicy, HttpKeycloakMetadataSource,
    KeycloakIdentityVerifier, NodeDependencies, NodeServer, RotatingAuditSink,
};

use crate::commands::ServeCommand;
use crate::error::CliError;

pub(super) async fn run(command: ServeCommand) -> ExitCode {
    let settings = load_settings(&command).and_then(compose_server);
    let server = match settings {
        Ok(server) => server,
        Err(error) => {
            eprintln!("{}: {error}", error.code());
            return ExitCode::from(2);
        }
    };
    eprintln!("SynapseFlow node starting; press Ctrl+C to drain and stop.");
    server
        .serve_until(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_or_else(
            |_| {
                eprintln!("SYN-CLI-004: node server could not start");
                ExitCode::from(2)
            },
            |_| ExitCode::SUCCESS,
        )
}

fn compose_server(settings: ServeSettings) -> Result<NodeServer, CliError> {
    let node_settings = settings.node;
    if node_settings.model_policy.allowed_models.len() != 1
        || !node_settings
            .model_policy
            .allowed_models
            .contains(&settings.model)
    {
        return Err(CliError::NodeConfigurationInvalid);
    }
    let identity_source =
        HttpKeycloakMetadataSource::new().map_err(|_| CliError::NodeConfigurationInvalid)?;
    let identity = Arc::new(KeycloakIdentityVerifier::new(
        node_settings.keycloak.clone(),
        identity_source,
    ));
    let audit = Arc::new(
        RotatingAuditSink::open(node_settings.audit.clone())
            .map_err(|_| CliError::NodeConfigurationUnavailable)?,
    );
    let telemetry = Arc::new(BoundedTelemetrySink::new(
        node_settings.telemetry.queue_capacity,
        Arc::new(StderrTelemetryExporter),
    ));
    let model_policy = Arc::new(ConfiguredModelAccessPolicy::new(
        node_settings.model_policy.allowed_models.clone(),
    ));
    let state = Arc::new(
        SqliteNodeState::open(
            &node_settings.state.database_path,
            SqliteNodeStateSettings {
                max_concurrent_sessions: node_settings.admission.max_concurrent_sessions,
                max_sessions_per_principal: node_settings.admission.max_sessions_per_principal,
            },
        )
        .map_err(|_| CliError::NodeConfigurationUnavailable)?,
    );
    let sessions = Arc::new(GenerationSessionManager::new(
        state.clone(),
        model_policy.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        audit.clone(),
    ));
    let generation = Arc::new(
        crate::runtime::build_node_generation_orchestrator(
            &settings.model,
            settings.config,
            audit.clone(),
        )
        .map_err(|_| CliError::NodeConfigurationUnavailable)?,
    );
    let execution = Arc::new(SessionExecutionService::new(sessions.clone(), generation));
    NodeServer::with_dependencies(
        node_settings,
        NodeDependencies {
            identity,
            audit,
            telemetry,
            model_policy,
            sessions,
            execution,
        },
    )
    .map_err(|_| CliError::NodeConfigurationInvalid)
}
