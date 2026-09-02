use std::fs::File;
use std::path::Path;

use candle_core::quantized::gguf_file::{Content, Value};
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::Device;
use synapseflow_domain::{DomainError, DomainResult};

use crate::runtime::LoomModelLayout;

/// Metadata needed to load and execute one Llama GGUF artifact.
#[derive(Clone, Debug)]
pub(crate) struct LlamaLayout {
    pub block_count: u32,
    pub embedding_width: u32,
    pub vocabulary_size: u32,
    pub attention_heads: u32,
    pub key_value_heads: u32,
    pub rope_dimension: u32,
    pub context_limit: u32,
    pub rms_epsilon: f32,
    pub rope_frequency_base: f32,
}

/// A verified artifact reader. Candle data stays contained in Loom.
pub(crate) struct LoomArchive {
    content: Content,
    reader: File,
    layout: LlamaLayout,
}

impl LoomArchive {
    pub(crate) fn open(path: &Path) -> DomainResult<Self> {
        let mut reader = File::open(path).map_err(|_| DomainError::ArtifactUnavailable)?;
        let content = Content::read(&mut reader).map_err(|_| DomainError::BackendIncompatible)?;
        let layout = LlamaLayout {
            block_count: required_u32(&content, "llama.block_count")?,
            embedding_width: required_u32(&content, "llama.embedding_length")?,
            vocabulary_size: optional_u32(&content, "llama.vocab_size")
                .unwrap_or(embedded_tokenizer_size(&content)?),
            attention_heads: required_u32(&content, "llama.attention.head_count")?,
            key_value_heads: required_u32(&content, "llama.attention.head_count_kv")?,
            rope_dimension: required_u32(&content, "llama.rope.dimension_count")?,
            context_limit: required_u32(&content, "llama.context_length")?,
            rms_epsilon: required_f32(&content, "llama.attention.layer_norm_rms_epsilon")?,
            rope_frequency_base: optional_f32(&content, "llama.rope.freq_base").unwrap_or(10_000.0),
        };
        if required_string(&content, "general.architecture")? != "llama"
            || layout.block_count == 0
            || layout.embedding_width == 0
            || layout.vocabulary_size == 0
            || layout.attention_heads == 0
            || layout.key_value_heads == 0
            || layout.rope_dimension == 0
            || layout.context_limit == 0
            || !layout
                .embedding_width
                .is_multiple_of(layout.attention_heads)
            || !layout
                .attention_heads
                .is_multiple_of(layout.key_value_heads)
            || layout.rope_dimension > layout.embedding_width / layout.attention_heads
            || !layout.rms_epsilon.is_finite()
            || layout.rms_epsilon <= 0.0
            || !layout.rope_frequency_base.is_finite()
            || layout.rope_frequency_base <= 0.0
        {
            return Err(DomainError::BackendIncompatible);
        }
        validate_embedded_tokenizer(&content, layout.vocabulary_size)?;
        Ok(Self {
            content,
            reader,
            layout,
        })
    }

    pub(crate) fn model_layout(&self) -> LoomModelLayout {
        LoomModelLayout {
            layer_count: self.layout.block_count,
            activation_width: self.layout.embedding_width,
            vocabulary_size: self.layout.vocabulary_size,
        }
    }

    pub(crate) fn tokenizer_tokens(&self) -> DomainResult<Vec<String>> {
        embedded_tokenizer(&self.content)?
            .iter()
            .map(|token| {
                token
                    .to_string()
                    .cloned()
                    .map_err(|_| DomainError::TokenizerFailure)
            })
            .collect()
    }

    pub(crate) fn layout(&self) -> &LlamaLayout {
        &self.layout
    }

    /// Loads one matrix encoded in a GGUF quantized matrix format accepted by Loom.
    pub(crate) fn quantized_matrix(&mut self, name: &str) -> DomainResult<QTensor> {
        let tensor = self
            .content
            .tensor(&mut self.reader, name, &Device::Cpu)
            .map_err(|_| DomainError::BackendIncompatible)?;
        if !matches!(tensor.dtype(), GgmlDType::Q5K | GgmlDType::Q6K) {
            return Err(DomainError::BackendIncompatible);
        }
        Ok(tensor)
    }

    pub(crate) fn norm(&mut self, name: &str) -> DomainResult<candle_core::Tensor> {
        self.content
            .tensor(&mut self.reader, name, &Device::Cpu)
            .and_then(|tensor| tensor.dequantize(&Device::Cpu))
            .map_err(|_| DomainError::BackendIncompatible)
    }
}

fn metadata<'a>(content: &'a Content, key: &str) -> DomainResult<&'a Value> {
    content
        .metadata
        .get(key)
        .ok_or(DomainError::BackendIncompatible)
}

fn required_string<'a>(content: &'a Content, key: &str) -> DomainResult<&'a str> {
    metadata(content, key)?
        .to_string()
        .map(String::as_str)
        .map_err(|_| DomainError::BackendIncompatible)
}

fn required_u32(content: &Content, key: &str) -> DomainResult<u32> {
    metadata(content, key)?
        .to_u32()
        .map_err(|_| DomainError::BackendIncompatible)
}

fn required_f32(content: &Content, key: &str) -> DomainResult<f32> {
    metadata(content, key)?
        .to_f32()
        .map_err(|_| DomainError::BackendIncompatible)
}

fn optional_f32(content: &Content, key: &str) -> Option<f32> {
    content
        .metadata
        .get(key)
        .and_then(|value| value.to_f32().ok())
}

fn optional_u32(content: &Content, key: &str) -> Option<u32> {
    content
        .metadata
        .get(key)
        .and_then(|value| value.to_u32().ok())
}

fn validate_embedded_tokenizer(content: &Content, vocabulary_size: u32) -> DomainResult<()> {
    let tokens = embedded_tokenizer(content)?;
    if tokens.len() != vocabulary_size as usize
        || tokens
            .iter()
            .any(|token| !matches!(token, Value::String(_)))
    {
        return Err(DomainError::BackendIncompatible);
    }
    Ok(())
}

fn embedded_tokenizer(content: &Content) -> DomainResult<&Vec<Value>> {
    metadata(content, "tokenizer.ggml.tokens")?
        .to_vec()
        .map_err(|_| DomainError::BackendIncompatible)
}

fn embedded_tokenizer_size(content: &Content) -> DomainResult<u32> {
    u32::try_from(embedded_tokenizer(content)?.len()).map_err(|_| DomainError::BackendIncompatible)
}
