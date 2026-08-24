use std::sync::Arc;
use std::time::Instant;

use synapseflow_domain::execution::{
    DecodedFrame, FrameEnvelope, FrameMessageType, FrameProtocolVersion, TensorDescriptor,
    TensorDtype,
};
use synapseflow_domain::{
    DomainError, DomainResult, ExecutionStrategy, FrameCodec, FrameCompression,
};
use synapseflow_ports::{
    ExecutionCancellation, ShardExecutionBackend, ShardExecutionOutput, ShardExecutionRequest,
    VerifiedModel,
};

use crate::compatibility::{is_final_stage, validate_model};
use crate::input::{parse_stage_input, position_extension, StageInput};
use crate::loom::LoomEngine;
use crate::runtime::{LoomExecutionOutput, LoomExecutionRequest, LoomExecutor};

/// Llama-specific adapter for the approved `layer_range_v1` strategy only.
pub struct LoomBackend {
    executor: Arc<dyn LoomExecutor>,
}

impl LoomBackend {
    /// Constructs the production adapter with Loom's CPU execution engine.
    pub fn new() -> Self {
        Self::with_executor(Arc::new(LoomEngine::new()))
    }

    /// Constructs the adapter around a Loom execution engine.
    pub fn with_executor(executor: Arc<dyn LoomExecutor>) -> Self {
        Self { executor }
    }

    fn execute_validated(
        &self,
        model: &VerifiedModel,
        request: &ShardExecutionRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<ShardExecutionOutput> {
        if cancellation.is_cancelled() {
            return Err(DomainError::SessionCancelled);
        }
        validate_model(&model.manifest)?;
        request.validate_for(model)?;

        let shard = request.requirements.shard();
        let final_stage = is_final_stage(&model.manifest, shard)?;
        let artifact = model.artifact_path(shard.artifact_id())?;
        let layout = self.executor.inspect(artifact)?;
        let declared_layers = model
            .manifest
            .execution_plan
            .as_ref()
            .ok_or(DomainError::ShardPlanInvalid)?
            .total_layers();
        if layout.layer_count != declared_layers
            || layout.activation_width == 0
            || layout.vocabulary_size == 0
        {
            return Err(DomainError::BackendIncompatible);
        }

        let input = parse_stage_input(&request.input, shard.layer_range().start() == 0)?;
        validate_input_width(&input, layout.activation_width)?;
        let started = Instant::now();
        let loom_request = LoomExecutionRequest {
            model: request.target.model.clone(),
            session_id: request.input.envelope.session_id.clone(),
            declared_range: shard.layer_range(),
            input,
            final_stage,
            remaining_deadline: request.remaining_deadline,
        };
        let loom_output = self
            .executor
            .execute(artifact, &loom_request, cancellation)?;
        if cancellation.is_cancelled() {
            return Err(DomainError::SessionCancelled);
        }
        if started.elapsed() >= request.remaining_deadline.duration() {
            return Err(DomainError::DeadlineExceeded);
        }

        self.encode_output(
            request,
            loom_output,
            final_stage,
            layout.activation_width,
            layout.vocabulary_size,
        )
    }

    fn encode_output(
        &self,
        request: &ShardExecutionRequest,
        loom_output: LoomExecutionOutput,
        final_stage: bool,
        activation_width: u32,
        vocabulary_size: u32,
    ) -> DomainResult<ShardExecutionOutput> {
        let (values, output_target, terminal, output_shape) = match (final_stage, loom_output) {
            (
                false,
                LoomExecutionOutput::Boundary {
                    activations,
                    token_count,
                },
            ) if activations.len() == token_count.saturating_mul(activation_width as usize) => (
                activations,
                request
                    .next_target
                    .clone()
                    .ok_or(DomainError::ShardPlanInvalid)?,
                false,
                vec![
                    u32::try_from(token_count).map_err(|_| DomainError::FrameBoundsExceeded)?,
                    activation_width,
                ],
            ),
            (true, LoomExecutionOutput::FinalLogits(values))
                if values.len() == vocabulary_size as usize =>
            {
                (values, request.target.clone(), true, vec![vocabulary_size])
            }
            _ => return Err(DomainError::GenerationFailed),
        };
        if values.iter().any(|value| !value.is_finite()) {
            return Err(DomainError::GenerationFailed);
        }
        let payload = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        let position = stage_position(&request.input)?;
        let envelope = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Data,
            request.input.envelope.session_id.clone(),
            request.input.envelope.stream_id,
            request.input.envelope.sequence.next()?,
            output_target,
            Some(TensorDescriptor::new(TensorDtype::F32, output_shape)?),
            request.remaining_deadline,
        )?;
        let bytes = FrameCodec::encode_with_extensions(
            &envelope,
            &payload,
            FrameCompression::None,
            request.input.trace_id.as_ref(),
            &[position_extension(position)?],
        )?;
        let frame = FrameCodec::decode(&bytes)?;
        if terminal {
            Ok(ShardExecutionOutput::FinalLogits(frame))
        } else {
            Ok(ShardExecutionOutput::Boundary(frame))
        }
    }
}

impl Default for LoomBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ShardExecutionBackend for LoomBackend {
    fn supports(&self, strategy: &ExecutionStrategy) -> bool {
        strategy.is_layer_range()
    }

    fn execute(
        &self,
        model: &VerifiedModel,
        request: &ShardExecutionRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<ShardExecutionOutput> {
        self.execute_validated(model, request, cancellation)
    }
}

fn validate_input_width(input: &StageInput, activation_width: u32) -> DomainResult<()> {
    match input {
        StageInput::TokenIds { token_ids, .. } if !token_ids.is_empty() => Ok(()),
        StageInput::Boundary {
            activations,
            token_count,
            ..
        } if activations.len() == token_count.saturating_mul(activation_width as usize)
            && activations.iter().all(|value| value.is_finite()) =>
        {
            Ok(())
        }
        _ => Err(DomainError::FrameInvalid),
    }
}

fn stage_position(frame: &DecodedFrame) -> DomainResult<u64> {
    match parse_stage_input(
        frame,
        frame
            .envelope
            .tensor
            .as_ref()
            .is_some_and(|tensor| tensor.dtype == TensorDtype::U32),
    )? {
        StageInput::TokenIds { position_start, .. }
        | StageInput::Boundary { position_start, .. } => Ok(position_start),
    }
}
