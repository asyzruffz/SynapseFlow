use std::{fs, io::Write, path::Path};

use synapseflow_domain::{ArtifactDescriptor, DomainError, DomainResult, ModelReference};

/// Stores only safe, verified cache provenance; neither paths nor source contents are recorded.
pub(super) fn write_metadata(
    staged: &Path,
    destination: &Path,
    reference: &ModelReference,
    artifact: &ArtifactDescriptor,
) -> DomainResult<()> {
    let mut file = fs::File::create(staged).map_err(|_| DomainError::CacheFailure)?;
    write!(
        file,
        "reference={}\nartifact_id={}\ncontent_sha256={}\nsize_bytes={}\n",
        reference.as_str(),
        artifact.id.as_str(),
        artifact.content_sha256,
        artifact.size_bytes,
    )
    .map_err(|_| DomainError::CacheFailure)?;
    file.flush().map_err(|_| DomainError::CacheFailure)?;

    if destination.exists() {
        fs::remove_file(staged).map_err(|_| DomainError::CacheFailure)?;
        return Ok(());
    }
    fs::rename(staged, destination).map_err(|_| DomainError::CacheFailure)
}
