//! Filesystem adapters for provisioned manifests and verified content-addressed artifacts.

mod artifact_cache;
mod manifest_registry;

pub use artifact_cache::ContentAddressedArtifactStore;
pub use manifest_registry::ProvisionedManifestRegistry;
