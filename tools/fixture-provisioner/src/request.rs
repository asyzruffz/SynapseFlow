use std::path::PathBuf;

/// User-controlled locations and public metadata for one fixture manifest.
pub(crate) struct ProvisioningRequest {
    pub(crate) artifact_path: PathBuf,
    pub(crate) artifact_uri: String,
    pub(crate) signing_key_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) model_id: String,
    pub(crate) model_version: String,
    pub(crate) publisher_key_id: String,
    pub(crate) license: String,
    pub(crate) provenance: String,
}
