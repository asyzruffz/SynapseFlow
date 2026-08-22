use std::{fs, path::Path};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use synapseflow_domain::{
    ModelManifest, ModelReference, TrustStore, TrustedPublisher, MANIFEST_SCHEMA_VERSION,
};

use crate::{artifact::ArtifactFingerprint, request::ProvisioningRequest};

/// Canonical signed-manifest data safe to persist and report to the operator.
pub(crate) struct SignedManifest {
    pub(crate) document: Vec<u8>,
    pub(crate) reference: String,
    pub(crate) public_key_base64url: String,
}

impl SignedManifest {
    pub(crate) fn create(
        request: &ProvisioningRequest,
        artifact: &ArtifactFingerprint,
    ) -> Result<Self, String> {
        let signing_key = read_signing_key(&request.signing_key_path)?;
        let public_key_base64url = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
        let unsigned = UnsignedFixtureManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            model_id: &request.model_id,
            model_version: &request.model_version,
            format: "gguf",
            architecture: "llama",
            quantization: "Q5_K_M",
            tokenizer: TokenizerWire {
                kind: "embedded",
                model: "llama",
            },
            artifacts: [ArtifactWire {
                artifact_id: "weights",
                uri: &request.artifact_uri,
                content_sha256: format!("sha256:{}", artifact.content_sha256),
                size_bytes: artifact.size_bytes,
            }],
            publisher_key_id: &request.publisher_key_id,
            license: &request.license,
            provenance: &request.provenance,
        };
        let canonical_unsigned = canonicalize(&unsigned)?;
        let signature = format!(
            "base64url:{}",
            URL_SAFE_NO_PAD.encode(signing_key.sign(&canonical_unsigned).to_bytes())
        );
        let manifest = FixtureManifest {
            schema_version: unsigned.schema_version,
            model_id: unsigned.model_id,
            model_version: unsigned.model_version,
            format: unsigned.format,
            architecture: unsigned.architecture,
            quantization: unsigned.quantization,
            tokenizer: unsigned.tokenizer,
            artifacts: unsigned.artifacts,
            publisher_key_id: unsigned.publisher_key_id,
            license: unsigned.license,
            provenance: unsigned.provenance,
            signature,
        };
        let document = canonicalize(&manifest)?;
        let reference = ModelReference::parse(format!(
            "registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:{}",
            hex_lower(&Sha256::digest(&document))
        ))
        .map_err(|error| format!("generated reference is invalid: {error}"))?;

        verify_with_domain_parser(
            &reference,
            &document,
            &request.publisher_key_id,
            &public_key_base64url,
        )?;

        Ok(Self {
            document,
            reference: reference.as_str().to_owned(),
            public_key_base64url,
        })
    }
}

#[derive(Serialize)]
struct FixtureManifest<'a> {
    schema_version: u16,
    model_id: &'a str,
    model_version: &'a str,
    format: &'a str,
    architecture: &'a str,
    quantization: &'a str,
    tokenizer: TokenizerWire,
    artifacts: [ArtifactWire<'a>; 1],
    publisher_key_id: &'a str,
    license: &'a str,
    provenance: &'a str,
    signature: String,
}

#[derive(Serialize)]
struct UnsignedFixtureManifest<'a> {
    schema_version: u16,
    model_id: &'a str,
    model_version: &'a str,
    format: &'a str,
    architecture: &'a str,
    quantization: &'a str,
    tokenizer: TokenizerWire,
    artifacts: [ArtifactWire<'a>; 1],
    publisher_key_id: &'a str,
    license: &'a str,
    provenance: &'a str,
}

#[derive(Serialize)]
struct TokenizerWire {
    kind: &'static str,
    model: &'static str,
}

#[derive(Serialize)]
struct ArtifactWire<'a> {
    artifact_id: &'static str,
    uri: &'a str,
    content_sha256: String,
    size_bytes: u64,
}

fn read_signing_key(path: &Path) -> Result<SigningKey, String> {
    let encoded = fs::read_to_string(path)
        .map_err(|error| format!("cannot read signing key {}: {error}", path.display()))?;
    let encoded = encoded
        .trim()
        .strip_prefix("base64url:")
        .unwrap_or(encoded.trim());
    let key_bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| "signing key must be unpadded base64url".to_owned())?;
    let seed: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "signing key must decode to exactly 32 bytes".to_owned())?;
    Ok(SigningKey::from_bytes(&seed))
}

fn canonicalize(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|error| format!("cannot canonicalize manifest: {error}"))
}

fn verify_with_domain_parser(
    reference: &ModelReference,
    document: &[u8],
    publisher_key_id: &str,
    public_key_base64url: &str,
) -> Result<(), String> {
    let publisher = TrustedPublisher::new(publisher_key_id.to_owned(), public_key_base64url)
        .map_err(|error| format!("generated publisher key is invalid: {error}"))?;
    let trust_store = TrustStore::new([publisher])
        .map_err(|error| format!("cannot create trust store: {error}"))?;
    ModelManifest::parse_and_verify(reference.clone(), document, &trust_store)
        .map_err(|error| format!("generated manifest does not verify: {error}"))?;
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
