//! Reusable node API library.
//!
//! The `synapseflow` CLI owns server-process startup through its future `serve`
//! command. This crate contains no executable target.

mod error;
mod settings;

#[cfg(test)]
mod tests;

pub use error::NodeError;
pub use settings::{
    AdmissionSettings, AuditSettings, KeycloakSettings, ListenerSettings, NodeSettings, TlsSettings,
};
