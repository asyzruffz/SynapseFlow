use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use synapseflow_domain::{DomainError, DomainResult};

/// Short-lived exclusive lease preventing concurrent staging for the same cache key.
pub(super) struct CacheLease {
    path: PathBuf,
}

impl CacheLease {
    pub(super) fn acquire(path: PathBuf) -> DomainResult<Self> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| DomainError::CacheFailure)?;
        Ok(Self { path })
    }
}

impl Drop for CacheLease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
