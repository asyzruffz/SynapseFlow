use std::{env, fs, path::PathBuf};

use synapseflow_domain::{ModelReference, TrustStore, TrustedPublisher};

use super::vector::ReferenceVector;

const FIXTURE_ID: &str = "synapseflow-verified-local-tinyllama-q5km-v1";

/// Explicit local inputs for the ignored model-backed acceptance test.
pub(super) struct FixtureConfig {
    pub(super) artifact_path: PathBuf,
    pub(super) manifest_document: Vec<u8>,
    pub(super) reference: ModelReference,
    pub(super) trust_store: TrustStore,
    pub(super) expected_vector: Option<ReferenceVector>,
    pub(super) candidate_vector_path: Option<PathBuf>,
    pub(super) llama_cpp_revision: String,
}

impl FixtureConfig {
    pub(super) fn from_environment() -> Result<Self, String> {
        let artifact_path = required_path("SYNAPSEFLOW_FIXTURE_GGUF")?;
        let manifest_path = required_path("SYNAPSEFLOW_FIXTURE_MANIFEST")?;
        let manifest_document = fs::read(&manifest_path).map_err(|error| {
            format!(
                "cannot read SYNAPSEFLOW_FIXTURE_MANIFEST {}: {error}",
                manifest_path.display()
            )
        })?;
        let reference = ModelReference::parse(required_value("SYNAPSEFLOW_FIXTURE_REFERENCE")?)
            .map_err(|error| format!("SYNAPSEFLOW_FIXTURE_REFERENCE is invalid: {error}"))?;
        let public_key = required_value("SYNAPSEFLOW_FIXTURE_PUBLIC_KEY")?;
        let publisher = TrustedPublisher::new(
            "ed25519:synapseflow-fixture-2026-08".to_owned(),
            &public_key,
        )
        .map_err(|error| format!("SYNAPSEFLOW_FIXTURE_PUBLIC_KEY is invalid: {error}"))?;
        let trust_store = TrustStore::new([publisher])
            .map_err(|error| format!("cannot create trust store: {error}"))?;
        let expected_vector_path = optional_path("SYNAPSEFLOW_REFERENCE_VECTOR")?;
        let expected_vector = expected_vector_path
            .as_deref()
            .map(ReferenceVector::read)
            .transpose()?;
        let candidate_vector_path = optional_path("SYNAPSEFLOW_CANDIDATE_VECTOR")?;
        if expected_vector.is_none() && candidate_vector_path.is_none() {
            return Err(
                "set SYNAPSEFLOW_REFERENCE_VECTOR to verify an accepted vector, or set SYNAPSEFLOW_CANDIDATE_VECTOR to write a candidate vector"
                    .to_owned(),
            );
        }

        Ok(Self {
            artifact_path,
            manifest_document,
            reference,
            trust_store,
            expected_vector,
            candidate_vector_path,
            llama_cpp_revision: required_value("SYNAPSEFLOW_LLAMA_CPP_REVISION")?,
        })
    }

    pub(super) fn fixture_id() -> &'static str {
        FIXTURE_ID
    }
}

fn required_value(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("{name} must be set for the fixture acceptance test"))
}

fn required_path(name: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(required_value(name)?);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "{name} must name an existing file: {}",
            path.display()
        ))
    }
}

fn optional_path(name: &str) -> Result<Option<PathBuf>, String> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Ok(Some(PathBuf::from(value))),
        Ok(_) | Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(format!("cannot read {name}: {error}")),
    }
}
