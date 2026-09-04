//! Reusable node API library.
//!
//! The `synapseflow` CLI owns server-process startup through its future `serve`
//! command. This crate contains no executable target.

mod audit;
mod error;
mod keycloak;
mod model_policy;
mod server;
mod settings;
mod telemetry;
mod workflow_registry;

#[cfg(test)]
mod tests;

pub use audit::RotatingAuditSink;
pub use error::NodeError;
pub use keycloak::{
    HttpKeycloakMetadataSource, KeycloakIdentityVerifier, KeycloakMetadataError,
    KeycloakMetadataSource,
};
pub use model_policy::ConfiguredModelAccessPolicy;
pub use server::{NodeDependencies, NodeServer};
pub use settings::{
    AdmissionSettings, AuditSettings, KeycloakSettings, ListenerSettings, ModelPolicySettings,
    NodeProfile, NodeSettings, ShutdownSettings, StateSettings, TelemetrySettings, TlsSettings,
};
pub use telemetry::{BoundedTelemetrySink, TelemetryExporter, TelemetryRecord};
pub use workflow_registry::{NodeWorkflowRegistry, WorkflowSubscriber};
