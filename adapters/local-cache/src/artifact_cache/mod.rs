//! Atomic, content-addressed artifact cache backed by explicitly provisioned local files.

mod cache;
mod integrity;
mod lease;
mod metadata;
mod paths;
mod sources;
#[cfg(test)]
mod tests;

pub use cache::ContentAddressedArtifactStore;
