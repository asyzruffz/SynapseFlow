use std::num::NonZeroU32;

use encoding_rs::UTF_8;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel},
    sampling::LlamaSampler,
};
use synapseflow_domain::{
    DomainError, DomainResult, GeneratedToken, GenerationRequest, GenerationTerminal,
};
use synapseflow_ports::{ExecutionCancellation, GeneratedTokenSink, ModelBackend, VerifiedModel};

use crate::compatibility::{sampler_seed, validate_context, validate_manifest};

/// CPU-only runtime adapter backed by the pinned `llama-cpp-2` package.
pub struct LlamaCppBackend {
    backend: LlamaBackend,
}

impl LlamaCppBackend {
    pub fn new() -> DomainResult<Self> {
        LlamaBackend::init()
            .map(|backend| Self { backend })
            .map_err(|_| DomainError::BackendUnavailable)
    }

    fn load_model(&self, model: &VerifiedModel) -> DomainResult<LlamaModel> {
        validate_manifest(&model.manifest)?;
        let artifact = model.primary_artifact_path()?;
        LlamaModel::load_from_file(&self.backend, artifact, &Default::default())
            .map_err(|_| DomainError::BackendIncompatible)
    }

    fn generate_with_model(
        &self,
        model: &LlamaModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        let prompt_tokens = model
            .str_to_token(&request.prompt, AddBos::Always)
            .map_err(|_| DomainError::TokenizerFailure)?;
        let model_limit =
            usize::try_from(model.n_ctx_train()).map_err(|_| DomainError::BackendIncompatible)?;
        let context_limit = validate_context(prompt_tokens.len(), &request.policy, model_limit)?;
        let context_size =
            u32::try_from(context_limit).map_err(|_| DomainError::BackendIncompatible)?;
        let params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(context_size));
        let mut context = model
            .new_context(&self.backend, params)
            .map_err(|_| DomainError::BackendIncompatible)?;
        let mut batch = LlamaBatch::new(prompt_tokens.len(), 1);
        let last_prompt_token = prompt_tokens.len().saturating_sub(1);
        for (position, token) in prompt_tokens.iter().enumerate() {
            batch
                .add(
                    *token,
                    i32::try_from(position).map_err(|_| DomainError::GenerationPolicyInvalid)?,
                    &[0],
                    position == last_prompt_token,
                )
                .map_err(|_| DomainError::GenerationFailed)?;
        }
        context
            .decode(&mut batch)
            .map_err(|_| DomainError::GenerationFailed)?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_p(request.policy.top_p, 1),
            LlamaSampler::temp(request.policy.temperature),
            LlamaSampler::dist(sampler_seed(request.policy.seed)),
        ]);
        let mut decoder = UTF_8.new_decoder();
        let mut token_count = 0_usize;
        let mut position =
            i32::try_from(prompt_tokens.len()).map_err(|_| DomainError::GenerationPolicyInvalid)?;

        for _ in 0..request.policy.max_tokens {
            if cancellation.is_cancelled() {
                return Ok(GenerationTerminal::Cancelled);
            }
            if request.deadline_expired() {
                return Err(DomainError::DeadlineExceeded);
            }
            let token = sampler.sample(&context, batch.n_tokens() - 1);
            sampler.accept(token);
            if model.is_eog_token(token) {
                break;
            }
            let text = model
                .token_to_piece(token, &mut decoder, true, None)
                .map_err(|_| DomainError::TokenizerFailure)?;
            tokens.emit_token(GeneratedToken {
                id: u32::try_from(token.0).map_err(|_| DomainError::TokenizerFailure)?,
                text,
            })?;
            token_count = token_count
                .checked_add(1)
                .ok_or(DomainError::GenerationFailed)?;
            batch.clear();
            batch
                .add(token, position, &[0], true)
                .map_err(|_| DomainError::GenerationFailed)?;
            context
                .decode(&mut batch)
                .map_err(|_| DomainError::GenerationFailed)?;
            position = position
                .checked_add(1)
                .ok_or(DomainError::GenerationPolicyInvalid)?;
        }
        Ok(GenerationTerminal::Completed { token_count })
    }
}

impl ModelBackend for LlamaCppBackend {
    fn generate(
        &self,
        verified_model: &VerifiedModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        let model = self.load_model(verified_model)?;
        self.generate_with_model(&model, request, cancellation, tokens)
    }
}
