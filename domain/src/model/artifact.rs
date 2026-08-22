use crate::{DomainError, DomainResult};

/// Stable artifact identifier within one immutable manifest.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactId(String);

impl ArtifactId {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.is_empty() || value.len() > 128 {
            return Err(DomainError::ManifestInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Verified immutable artifact metadata supplied by a manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactDescriptor {
    pub id: ArtifactId,
    pub content_sha256: String,
    pub size_bytes: u64,
}
