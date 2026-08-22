use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};
use synapseflow_domain::{ArtifactDescriptor, DomainError, DomainResult};

/// Copies a provisioned source into staging while enforcing its signed size and hash.
pub(super) fn copy_verified(
    source: &Path,
    staged: &Path,
    artifact: &ArtifactDescriptor,
    max_artifact_bytes: u64,
) -> DomainResult<()> {
    if artifact.size_bytes > max_artifact_bytes {
        return Err(DomainError::ArtifactIntegrity);
    }

    let mut input = File::open(source).map_err(|_| DomainError::ArtifactUnavailable)?;
    let mut output = File::create(staged).map_err(|_| DomainError::CacheFailure)?;
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 8192];

    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|_| DomainError::ArtifactUnavailable)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or(DomainError::ArtifactIntegrity)?;
        if total > artifact.size_bytes || total > max_artifact_bytes {
            return Err(DomainError::ArtifactIntegrity);
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| DomainError::CacheFailure)?;
        digest.update(&buffer[..read]);
    }
    output.flush().map_err(|_| DomainError::CacheFailure)?;

    if total != artifact.size_bytes || !hash_matches(&digest.finalize(), artifact) {
        return Err(DomainError::ArtifactIntegrity);
    }
    Ok(())
}

/// Rechecks a cache object before treating it as a verified hit.
pub(super) fn verify_cached(path: &Path, artifact: &ArtifactDescriptor) -> DomainResult<()> {
    let mut input = File::open(path).map_err(|_| DomainError::CacheFailure)?;
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 8192];

    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|_| DomainError::CacheFailure)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or(DomainError::CacheFailure)?;
        digest.update(&buffer[..read]);
    }
    if total != artifact.size_bytes || !hash_matches(&digest.finalize(), artifact) {
        return Err(DomainError::ArtifactIntegrity);
    }
    Ok(())
}

fn hash_matches(observed: &[u8], artifact: &ArtifactDescriptor) -> bool {
    let expected = artifact
        .content_sha256
        .strip_prefix("sha256:")
        .unwrap_or("");
    let mut encoded = String::with_capacity(observed.len() * 2);
    for byte in observed {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded == expected
}
