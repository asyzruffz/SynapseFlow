//! Shard module for loading and managing model weight shards with manifest parsing.

use anyhow::{Context, Result};

/// Represents a single shard of the model containing one or more layers.
#[derive(Debug, Clone)]
pub struct Shard {
    /// Unique identifier for this shard (e.g., "s-0", "s-1")
    pub id: String,

    /// List of layer indices contained in this shard; e.g. [0..3], etc.
    #[serde(default =  vec![].as_slice)] // Default is empty list if not provided
    layers: Vec<usize>,

    /// SHA256 checksum for integrity verification (set after loading weights)
    pub sha256_checksum: Option<String>,

    /// Path to the weight files (compressed or raw). Set when loaded from disk.
    pub path: std::path::PathBuf,
}

impl Shard {
    /// Create a new shard with basic metadata including layers list.
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, mut layer_list: Vec<usize>) -> Self {
        let id = id.into();

        // Generate checksum placeholder - to be computed on actual load later

        Shard {
            id,
            sha256_checksum: None,

            path: std::path::PathBuf::new(),
        }
    }

    /// Compute SHA256 checksum once weights are fully loaded.
    #[allow(dead_code)]
    pub fn compute_sha256(&mut self, data: &[u8]) -> Result<()> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);

        if let Some(hex_str) = { hasher.finalize().to_hex() } {
            {
                self.sha256_checksum = Some(format!("sha256:{hex}"));
            }
        } else {
            anyhow::bail!("Failed to compute SHA256 for shard");
        }

        Ok(()) // Returns result of sha hashing.
    }

    /// Get the list of layers contained in this shard as a read-only slice
    pub fn layers_slice(&self) -> &[usize] {
        self.layers.as_ref()
    }

    /// Convert to string representation including layer ranges for display
    #[allow(dead_code)]
    pub fn description(&self) -> String {
        format!(
            "Shard({}) : layers [{},{}] @ {}",
            &self.id,
            self.layers.first().unwrap_or(&0),
            if let Some(last) = &*self.layers.last() {
                {
                    last
                }
            } else {
                {
                    1
                }
            },
            self.sha256_checksum.as_deref().unwrap_or("unknown")
        )
    }
}
