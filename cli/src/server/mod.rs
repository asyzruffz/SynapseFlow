mod configuration;
mod telemetry;

use configuration::load_settings;
use telemetry::StderrTelemetryExporter;

use std::{process::ExitCode, sync::Arc};

use synapseflow_node::{
    BoundedTelemetrySink, ConfiguredModelAccessPolicy, HttpKeycloakMetadataSource,
    KeycloakIdentityVerifier, NodeDependencies, NodeServer, NodeSettings, RotatingAuditSink,
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

fn compose_server(settings: NodeSettings) -> Result<NodeServer, CliError> {
    let identity_source =
        HttpKeycloakMetadataSource::new().map_err(|_| CliError::NodeConfigurationInvalid)?;
    let identity = Arc::new(KeycloakIdentityVerifier::new(
        settings.keycloak.clone(),
        identity_source,
    ));
    let audit = Arc::new(
        RotatingAuditSink::open(settings.audit.clone())
            .map_err(|_| CliError::NodeConfigurationUnavailable)?,
    );
    let telemetry = Arc::new(BoundedTelemetrySink::new(
        settings.telemetry.queue_capacity,
        Arc::new(StderrTelemetryExporter),
    ));
    let model_policy = Arc::new(ConfiguredModelAccessPolicy::new(
        settings.model_policy.allowed_models.clone(),
    ));
    NodeServer::with_dependencies(
        settings,
        NodeDependencies {
            identity,
            audit,
            telemetry,
            model_policy,
        },
    )
    .map_err(|_| CliError::NodeConfigurationInvalid)
}
