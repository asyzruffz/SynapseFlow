use std::path::{Path, PathBuf};

use synapseflow_domain::ArtifactDescriptor;

pub(super) struct CachePaths<'a> {
    root: &'a Path,
}

impl<'a> CachePaths<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self { root }
    }

    pub(super) fn objects(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub(super) fn staging(&self) -> PathBuf {
        self.root.join("staging")
    }

    pub(super) fn leases(&self) -> PathBuf {
        self.root.join("leases")
    }

    pub(super) fn metadata(&self) -> PathBuf {
        self.root.join("metadata")
    }

    pub(super) fn object(&self, artifact: &ArtifactDescriptor) -> PathBuf {
        self.objects().join(hash(artifact))
    }

    pub(super) fn lease(&self, artifact: &ArtifactDescriptor) -> PathBuf {
        self.leases().join(format!("{}.lease", hash(artifact)))
    }

    pub(super) fn metadata_file(&self, artifact: &ArtifactDescriptor) -> PathBuf {
        self.metadata().join(format!("{}.meta", hash(artifact)))
    }

    pub(super) fn staged_artifact(&self, artifact: &ArtifactDescriptor, suffix: u64) -> PathBuf {
        self.staging()
            .join(format!("{}.{}.part", hash(artifact), suffix))
    }

    pub(super) fn staged_metadata(&self, artifact: &ArtifactDescriptor, suffix: u64) -> PathBuf {
        self.staging()
            .join(format!("{}.{}.meta.part", hash(artifact), suffix))
    }
}

fn hash(artifact: &ArtifactDescriptor) -> &str {
    artifact
        .content_sha256
        .strip_prefix("sha256:")
        .unwrap_or("")
}
