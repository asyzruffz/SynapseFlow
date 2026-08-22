//! Wire-only manifest representations used for decoding and signing.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManifestWire {
    pub(super) schema_version: u16,
    pub(super) model_id: String,
    pub(super) model_version: String,
    pub(super) format: String,
    pub(super) architecture: String,
    pub(super) quantization: String,
    pub(super) tokenizer: TokenizerWire,
    pub(super) artifacts: Vec<ArtifactWire>,
    pub(super) publisher_key_id: String,
    pub(super) license: String,
    pub(super) provenance: String,
    pub(super) signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenizerWire {
    pub(super) kind: String,
    pub(super) model: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactWire {
    pub(super) artifact_id: String,
    pub(super) uri: String,
    pub(super) content_sha256: String,
    pub(super) size_bytes: u64,
}

#[derive(Serialize)]
pub(super) struct UnsignedManifest<'a> {
    schema_version: u16,
    model_id: &'a str,
    model_version: &'a str,
    format: &'a str,
    architecture: &'a str,
    quantization: &'a str,
    tokenizer: &'a TokenizerWire,
    artifacts: &'a [ArtifactWire],
    publisher_key_id: &'a str,
    license: &'a str,
    provenance: &'a str,
}

impl<'a> From<&'a ManifestWire> for UnsignedManifest<'a> {
    fn from(wire: &'a ManifestWire) -> Self {
        Self {
            schema_version: wire.schema_version,
            model_id: &wire.model_id,
            model_version: &wire.model_version,
            format: &wire.format,
            architecture: &wire.architecture,
            quantization: &wire.quantization,
            tokenizer: &wire.tokenizer,
            artifacts: &wire.artifacts,
            publisher_key_id: &wire.publisher_key_id,
            license: &wire.license,
            provenance: &wire.provenance,
        }
    }
}
