use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use synapseflow_domain::{ArtifactDescriptor, DomainError, DomainResult, ModelManifest};
use synapseflow_ports::{
    ArtifactStore, CacheEntryState, CachedArtifactInspection, ModelCacheInspection, VerifiedModel,
};

use super::{
    integrity::{copy_verified, verify_cached},
    lease::CacheLease,
    metadata::write_metadata,
    paths::CachePaths,
    sources::ProvisionedArtifactSources,
};

static STAGING_SUFFIX: AtomicU64 = AtomicU64::new(0);

/// A content-addressed artifact cache populated only from explicitly provisioned local files.
pub struct ContentAddressedArtifactStore {
    root: PathBuf,
    max_artifact_bytes: u64,
    sources: ProvisionedArtifactSources,
}

impl ContentAddressedArtifactStore {
    pub fn new(root: PathBuf, max_artifact_bytes: u64) -> DomainResult<Self> {
        if max_artifact_bytes == 0 {
            return Err(DomainError::CacheFailure);
        }
        let store = Self {
            root,
            max_artifact_bytes,
            sources: ProvisionedArtifactSources::default(),
        };
        store.create_layout()?;
        Ok(store)
    }

    /// Registers the only local source that may satisfy a declared HTTPS artifact URI.
    pub fn register_provisioned_source(
        &mut self,
        uri: String,
        source: PathBuf,
    ) -> DomainResult<()> {
        self.sources.insert(uri, source)
    }

    /// Retains the active model's object and removes inactive complete objects and orphaned staging.
    pub fn cleanup_except(&self, active: &ModelManifest) -> DomainResult<()> {
        if !active.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        let paths = CachePaths::new(&self.root);
        let active_names = active
            .artifacts
            .iter()
            .filter_map(|artifact| artifact.content_sha256.strip_prefix("sha256:"))
            .collect::<Vec<_>>();
        self.remove_unretained(&paths.objects(), &active_names)?;
        self.remove_unretained(&paths.metadata(), &active_names)?;
        self.remove_all_files(&paths.staging())
    }

    fn create_layout(&self) -> DomainResult<()> {
        let paths = CachePaths::new(&self.root);
        for directory in [
            paths.objects(),
            paths.staging(),
            paths.leases(),
            paths.metadata(),
        ] {
            fs::create_dir_all(directory).map_err(|_| DomainError::CacheFailure)?;
        }
        Ok(())
    }

    fn cache_artifact(
        &self,
        manifest: &ModelManifest,
        artifact: &ArtifactDescriptor,
    ) -> DomainResult<()> {
        let paths = CachePaths::new(&self.root);
        let destination = paths.object(artifact);
        if destination.exists() {
            return verify_cached(&destination, artifact);
        }

        let _lease = CacheLease::acquire(paths.lease(artifact))?;
        if destination.exists() {
            return verify_cached(&destination, artifact);
        }

        let suffix = STAGING_SUFFIX.fetch_add(1, Ordering::Relaxed);
        let staged = paths.staged_artifact(artifact, suffix);
        let source = self.sources.get(artifact)?;
        let copied = copy_verified(source, &staged, artifact, self.max_artifact_bytes);
        if copied.is_err() {
            let _ = fs::remove_file(&staged);
            return copied;
        }
        if let Err(error) = fs::rename(&staged, &destination) {
            let _ = fs::remove_file(&staged);
            if destination.exists() {
                return verify_cached(&destination, artifact);
            }
            let _ = error;
            return Err(DomainError::CacheFailure);
        }

        write_metadata(
            &paths.staged_metadata(artifact, suffix),
            &paths.metadata_file(artifact),
            &manifest.reference,
            artifact,
        )
    }

    fn inspection(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection> {
        if !manifest.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        let paths = CachePaths::new(&self.root);
        let artifacts = manifest
            .artifacts
            .iter()
            .map(|artifact| {
                let object = paths.object(artifact);
                let state = if object.exists() {
                    verify_cached(&object, artifact)?;
                    CacheEntryState::Cached
                } else {
                    CacheEntryState::Missing
                };
                Ok(CachedArtifactInspection {
                    artifact_id: artifact.id.clone(),
                    content_sha256: artifact.content_sha256.clone(),
                    size_bytes: artifact.size_bytes,
                    state,
                })
            })
            .collect::<DomainResult<Vec<_>>>()?;
        Ok(ModelCacheInspection {
            reference: manifest.reference.clone(),
            publisher_key_id: manifest.publisher_key_id.clone(),
            license: manifest.license.clone(),
            provenance: manifest.provenance.clone(),
            artifacts,
        })
    }

    fn remove_unretained(&self, directory: &Path, retained_hashes: &[&str]) -> DomainResult<()> {
        for entry in fs::read_dir(directory).map_err(|_| DomainError::CacheFailure)? {
            let entry = entry.map_err(|_| DomainError::CacheFailure)?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let retained = retained_hashes
                .iter()
                .any(|hash| name.as_ref() == *hash || name.starts_with(&format!("{hash}.")));
            if !retained
                && entry
                    .file_type()
                    .map_err(|_| DomainError::CacheFailure)?
                    .is_file()
            {
                fs::remove_file(entry.path()).map_err(|_| DomainError::CacheFailure)?;
            }
        }
        Ok(())
    }

    fn remove_all_files(&self, directory: &Path) -> DomainResult<()> {
        for entry in fs::read_dir(directory).map_err(|_| DomainError::CacheFailure)? {
            let entry = entry.map_err(|_| DomainError::CacheFailure)?;
            if entry
                .file_type()
                .map_err(|_| DomainError::CacheFailure)?
                .is_file()
            {
                fs::remove_file(entry.path()).map_err(|_| DomainError::CacheFailure)?;
            }
        }
        Ok(())
    }
}

impl ArtifactStore for ContentAddressedArtifactStore {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        if !manifest.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        self.create_layout()?;
        for artifact in &manifest.artifacts {
            self.cache_artifact(manifest, artifact)?;
        }
        let artifact_paths = manifest
            .artifacts
            .iter()
            .map(|artifact| CachePaths::new(&self.root).object(artifact))
            .collect();
        VerifiedModel::with_cached_artifacts(manifest.clone(), artifact_paths)
    }

    fn inspect(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection> {
        self.inspection(manifest)
    }
}
