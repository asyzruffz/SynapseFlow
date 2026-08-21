use serde_json;
use std::path::PathBuf;

use candle_core::{DType, Device, Tensor, D};
use candle_nn::VarBuilder;
use candle_transformers::models::llama as llama_mod;
use candle_transformers::models::llama::LlamaConfig;
use tokenizers::Tokenizer;

use synapseflow_core::models::{loader::ModelLoader, source::ModelSource};

use crate::config::InferenceConfig;
use crate::error::{InferenceError, Result};
use crate::InferenceEngine;

#[derive(Clone)]
pub struct LlamaModel {
    llama: llama_mod::Llama,
    config: llama_mod::Config,
    device: Device,
    dtype: DType,
    tokenizer_path: Option<PathBuf>,
}

fn initialization_error(error: impl std::fmt::Display) -> InferenceError {
    InferenceError::Initialization {
        message: error.to_string(),
    }
}

fn generation_error(error: impl std::fmt::Display) -> InferenceError {
    InferenceError::Generation {
        message: error.to_string(),
    }
}

impl InferenceEngine for LlamaModel {
    fn initialize(source: ModelSource) -> Result<Self> {
        let model_files = ModelLoader::load(source)?;

        // Load model config
        let cfg: LlamaConfig = match model_files.config {
            Some(config_path) => {
                let data = std::fs::read_to_string(&config_path).map_err(initialization_error)?;
                serde_json::from_str(&data).map_err(initialization_error)?
            }
            None => {
                return Err(InferenceError::Initialization {
                    message: format!(
                        "missing config.json in model directory: {}",
                        model_files.dir.display()
                    ),
                });
            }
        };
        let config = cfg.into_config(false);

        // Device + dtype selection (CPU, f16 for weights by default)
        let device = Device::Cpu;
        let dtype = DType::F16;

        // Load vars via mmaped safetensors into a VarBuilder (unsafe helper provided by candle)
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&model_files.safetensors, dtype, &device)
                .map_err(initialization_error)?
        };

        // Build model from VarBuilder + config
        let llama = llama_mod::Llama::load(vb, &config).map_err(initialization_error)?;

        Ok(Self {
            llama,
            config,
            device,
            dtype,
            tokenizer_path: model_files.tokenizer,
        })
    }

    fn generate(
        &self,
        prompt: &str,
        config: InferenceConfig,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<()> {
        // Tokenize prompt using tokenizers crate if tokenizer.json present
        let tokenizer = match &self.tokenizer_path {
            Some(path) => Tokenizer::from_file(path).map_err(generation_error)?,
            None => {
                return Err(InferenceError::Generation {
                    message: "tokenizer.json not found in model directory".to_owned(),
                });
            }
        };

        let encoding = tokenizer.encode(prompt, true).map_err(generation_error)?;
        let mut tokens: Vec<u32> = encoding.get_ids().to_vec();

        // Prepare cache
        let mut cache = llama_mod::Cache::new(false, self.dtype, &self.config, &self.device)
            .map_err(generation_error)?;

        let mut generated = Vec::new();
        let mut index_pos = 0usize;

        for _ in 0..config.max_tokens {
            let context_size = tokens.len();
            let ctxt_vec: Vec<u32> = tokens[tokens.len().saturating_sub(context_size)..].to_vec();
            let input = Tensor::new(ctxt_vec.as_slice(), &self.device)
                .map_err(generation_error)?
                .unsqueeze(0)
                .map_err(generation_error)?;
            let logits = self
                .llama
                .forward(&input, index_pos, &mut cache)
                .map_err(generation_error)?;
            let logits = logits.squeeze(0).map_err(generation_error)?;

            // argmax sampling
            let next_token = logits
                .argmax(D::Minus1)
                .map_err(generation_error)?
                .to_scalar::<u32>()
                .map_err(generation_error)?;
            tokens.push(next_token);
            generated.push(next_token);
            index_pos += ctxt_vec.len();

            // Try to stop if EOS token matches config (optional)
            if let Some(eos) = &self.config.eos_token_id {
                match eos {
                    llama_mod::LlamaEosToks::Single(eos_id) if next_token == *eos_id => break,
                    llama_mod::LlamaEosToks::Multiple(v) if v.contains(&next_token) => break,
                    _ => {}
                }
            }
        }

        // Decode generated tokens to string
        let output = tokenizer
            .decode(&generated, true)
            .map_err(generation_error)?;
        on_token(&output);
        Ok(())
    }
}
