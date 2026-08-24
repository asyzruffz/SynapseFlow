//! Wire-only manifest representations used for decoding and signing.

use serde::{Deserialize, Serialize};

/// The immutable manifest documents accepted by the parser.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum ManifestDocument {
    Full(ManifestWire),
    Sharded(ShardedManifestWire),
}

impl Serialize for ManifestDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Full(wire) => wire.serialize(serializer),
            Self::Sharded(wire) => wire.serialize(serializer),
        }
    }
}

impl ManifestDocument {
    pub(super) fn publisher_key_id(&self) -> &str {
        match self {
            Self::Full(wire) => &wire.publisher_key_id,
            Self::Sharded(wire) => &wire.publisher_key_id,
        }
    }

    pub(super) fn signature(&self) -> &str {
        match self {
            Self::Full(wire) => &wire.signature,
            Self::Sharded(wire) => &wire.signature,
        }
    }
}

/// Schema-v1 wire format, retained unchanged for verified local inference.
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

/// Schema-v2 wire format for layer-range shard execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShardedManifestWire {
    pub(super) schema_version: u16,
    pub(super) model_id: String,
    pub(super) model_version: String,
    pub(super) format: String,
    pub(super) architecture: String,
    pub(super) quantization: String,
    pub(super) tokenizer: TokenizerWire,
    pub(super) artifacts: Vec<ArtifactWire>,
    pub(super) execution: ExecutionWire,
    pub(super) publisher_key_id: String,
    pub(super) license: String,
    pub(super) provenance: String,
    pub(super) signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExecutionWire {
    pub(super) strategy: String,
    pub(super) runtime_profile: String,
    pub(super) layer_count: u32,
    pub(super) shards: Vec<ShardWire>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShardWire {
    pub(super) shard_id: String,
    pub(super) artifact_id: String,
    pub(super) layer_start: u32,
    pub(super) layer_end_exclusive: u32,
    pub(super) minimum_replicas: u8,
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

/// Signature payload, excluding only the envelope signature field.
pub(super) enum UnsignedManifest<'a> {
    Full(UnsignedFullManifest<'a>),
    Sharded(UnsignedShardedManifest<'a>),
}

impl Serialize for UnsignedManifest<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Full(wire) => wire.serialize(serializer),
            Self::Sharded(wire) => wire.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
pub(super) struct UnsignedFullManifest<'a> {
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

#[derive(Serialize)]
pub(super) struct UnsignedShardedManifest<'a> {
    schema_version: u16,
    model_id: &'a str,
    model_version: &'a str,
    format: &'a str,
    architecture: &'a str,
    quantization: &'a str,
    tokenizer: &'a TokenizerWire,
    artifacts: &'a [ArtifactWire],
    execution: &'a ExecutionWire,
    publisher_key_id: &'a str,
    license: &'a str,
    provenance: &'a str,
}

impl<'a> From<&'a ManifestWire> for UnsignedManifest<'a> {
    fn from(wire: &'a ManifestWire) -> Self {
        Self::Full(UnsignedFullManifest {
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
        })
    }
}

impl<'a> From<&'a ShardedManifestWire> for UnsignedManifest<'a> {
    fn from(wire: &'a ShardedManifestWire) -> Self {
        Self::Sharded(UnsignedShardedManifest {
            schema_version: wire.schema_version,
            model_id: &wire.model_id,
            model_version: &wire.model_version,
            format: &wire.format,
            architecture: &wire.architecture,
            quantization: &wire.quantization,
            tokenizer: &wire.tokenizer,
            artifacts: &wire.artifacts,
            execution: &wire.execution,
            publisher_key_id: &wire.publisher_key_id,
            license: &wire.license,
            provenance: &wire.provenance,
        })
    }
}

impl<'a> From<&'a ManifestDocument> for UnsignedManifest<'a> {
    fn from(document: &'a ManifestDocument) -> Self {
        match document {
            ManifestDocument::Full(wire) => Self::from(wire),
            ManifestDocument::Sharded(wire) => Self::from(wire),
        }
    }
}
