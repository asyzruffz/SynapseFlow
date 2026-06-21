//! SynapseFlow Core Library
//!
//! This module handles:
//! - Model manifest parsing (GGUF/ONNX/HF formats)
//! - Manifest signature verification
//! - Local shard metadata storage via sled

pub mod model_loader;
mod shard_index; // Internal module - not part of public API to avoid breaking changes later

use anyhow::Result;

/// Version information for the core crate
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get a brief summary of what this library does.
pub fn description() -> &'static str {
    "Core component providing model loading, manifest handling, and shard indexing."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() -> anyhow::Result<()> {
        // TODO: Implement manifest parsing test
        Ok(())
    }

    #[test]
    fn test_signature_verification() -> anyhow::Result<()> {
        // TODO: Implement signature verification test
        Ok(())
    }
}
