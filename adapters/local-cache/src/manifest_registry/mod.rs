//! Explicit provisioning of signed manifest documents from allowed local sources.

mod registry;
#[cfg(test)]
mod tests;

pub use registry::ProvisionedManifestRegistry;
