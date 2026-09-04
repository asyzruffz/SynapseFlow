use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use synapseflow_domain::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, RemainingDeadline,
    RetryBudget, SessionId, StreamId, TensorDescriptor, TensorDtype,
};
use synapseflow_domain::{
    DecodedFrame, DomainError, DomainResult, ExecutionStrategy, FrameCodec, FrameCompression,
    GenerationRequest, GenerationTerminal,
};
use synapseflow_ports::{
    ExecutionCancellation, GeneratedTokenSink, ModelTokenizer, PeerDirectory,
    ShardExecutionBackend, ShardExecutionOutput, ShardExecutionRequest, ShardExecutionRequirements,
    ShardedGenerationRuntime, Transport, TransportReceipt, VerifiedModel, WorkerId,
};

use super::{IdempotencyKey, SessionConfiguration, SessionManager, ShardPlanner};

const DEFAULT_GENERATION_DEADLINE: Duration = Duration::from_secs(30);

/// Application-owned schema-v2 runtime that drives Loom stage adapters through
/// the canonical transport port.
pub struct LayerRangeShardedGenerationRuntime {
    planner: ShardPlanner,
    sessions: Arc<SessionManager>,
    transport: Arc<dyn Transport>,
    workers: BTreeMap<WorkerId, Arc<dyn ShardExecutionBackend>>,
    tokenizer: Arc<dyn ModelTokenizer>,
    next_request: AtomicU64,
}

impl LayerRangeShardedGenerationRuntime {
    pub fn new(
        directory: Arc<dyn PeerDirectory>,
        sessions: Arc<SessionManager>,
        transport: Arc<dyn Transport>,
        workers: BTreeMap<WorkerId, Arc<dyn ShardExecutionBackend>>,
        tokenizer: Arc<dyn ModelTokenizer>,
    ) -> Self {
        Self {
            planner: ShardPlanner::new(directory),
            sessions,
            transport,
            workers,
            tokenizer,
            next_request: AtomicU64::new(1),
        }
    }

    fn execute(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        let route = self.planner.plan(&model.manifest)?;
        let started = Instant::now();
        let deadline = request
            .remaining_deadline()
            .unwrap_or(DEFAULT_GENERATION_DEADLINE);
        let remaining_deadline = RemainingDeadline::new(deadline)?;
        let request_id = self.next_request.fetch_add(1, Ordering::Relaxed);
        let idempotency_key = IdempotencyKey::new(format!("loom-request-{request_id:016}"))?;
        let session_id = SessionId::new(format!("loom-session-{request_id:016}"))?;
        let configuration = SessionConfiguration {
            idempotency_key: idempotency_key.clone(),
            session_id: session_id.clone(),
            route,
            remaining_deadline,
            retry_budget: RetryBudget::new(1),
        };

        self.sessions.begin(configuration.clone())?;
        self.sessions.mark_planned(&idempotency_key)?;
        self.sessions.start(&idempotency_key)?;

        let result = self.generate_tokens(
            model,
            request,
            &configuration,
            started,
            deadline,
            cancellation,
            tokens,
        );
        let terminal = match &result {
            Ok(GenerationTerminal::Completed { .. }) => {
                self.sessions.complete(&idempotency_key).map(|_| ())
            }
            Ok(GenerationTerminal::Cancelled) | Err(DomainError::SessionCancelled) => self
                .sessions
                .cancel(&idempotency_key)
                .and_then(|_| self.sessions.finish_cancellation(&idempotency_key))
                .map(|_| ()),
            Err(_) => self.sessions.fail(&idempotency_key).map(|_| ()),
        };
        let released = self.release_session(model, &session_id);
        terminal?;
        released?;
        result
    }

    // These inputs are independently owned application contracts; combining
    // them would obscure the execution, deadline, and cancellation boundary.
    #[allow(clippy::too_many_arguments)]
    fn generate_tokens(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
        configuration: &SessionConfiguration,
        started: Instant,
        deadline: Duration,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        let mut input = self.tokenizer.encode(model, &request.prompt)?;
        let mut position_start = 0_u64;
        let mut sequence = FrameSequence::initial();
        let mut random_state = request.policy.seed;
        let mut token_count = 0_usize;

        for _ in 0..request.policy.max_tokens {
            if cancellation.is_cancelled() {
                return Ok(GenerationTerminal::Cancelled);
            }
            let logits = self.execute_stages(
                model,
                configuration,
                &input,
                position_start,
                sequence,
                remaining_deadline(started, deadline)?,
                cancellation,
            )?;
            sequence = logits.envelope.sequence.next()?;
            let token_id = sample_token(&logits.payload, &request.policy, &mut random_state)?;
            tokens.emit_token(self.tokenizer.decode(model, token_id)?)?;
            token_count = token_count
                .checked_add(1)
                .ok_or(DomainError::GenerationFailed)?;
            position_start = position_start
                .checked_add(
                    u64::try_from(input.len()).map_err(|_| DomainError::FrameBoundsExceeded)?,
                )
                .ok_or(DomainError::FrameBoundsExceeded)?;
            input = vec![token_id];
        }

        Ok(GenerationTerminal::Completed { token_count })
    }

    // Keep frame sequence, deadline, and cancellation explicit at the data-plane edge.
    #[allow(clippy::too_many_arguments)]
    fn execute_stages(
        &self,
        model: &VerifiedModel,
        configuration: &SessionConfiguration,
        token_ids: &[u32],
        position_start: u64,
        sequence: FrameSequence,
        remaining_deadline: RemainingDeadline,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<DecodedFrame> {
        let first = configuration
            .route
            .assignments
            .first()
            .ok_or(DomainError::ShardPlanInvalid)?;
        let payload = token_ids
            .iter()
            .flat_map(|token| token.to_le_bytes())
            .collect::<Vec<_>>();
        let envelope = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Data,
            configuration.session_id.clone(),
            StreamId::new(1)?,
            sequence,
            first.target.clone(),
            Some(TensorDescriptor::new(
                TensorDtype::U32,
                vec![u32::try_from(token_ids.len()).map_err(|_| DomainError::FrameBoundsExceeded)?],
            )?),
            remaining_deadline,
        )?;
        let bytes = FrameCodec::encode_with_extensions(
            &envelope,
            &payload,
            FrameCompression::None,
            None,
            &[synapseflow_domain::FrameExtension::new(
                2,
                position_start.to_be_bytes().to_vec(),
            )?],
        )?;
        let mut frame = FrameCodec::decode(&bytes)?;
        let mut source = first.primary.clone();

        for (index, assignment) in configuration.route.assignments.iter().enumerate() {
            let mut destination = assignment.primary.clone();
            let stage = self.deliver_and_execute(
                model,
                configuration,
                index,
                &source,
                &destination,
                frame.clone(),
                remaining_deadline,
                cancellation,
            );
            frame = match stage {
                Ok(frame) => frame,
                Err(DomainError::WorkerUnavailable) if index > 0 => {
                    self.sessions
                        .retry_from_latest_checkpoint(&configuration.idempotency_key, true)?;
                    destination = assignment
                        .replicas
                        .iter()
                        .find(|worker| {
                            **worker != assignment.primary
                                && self.workers.contains_key(*worker)
                                && self.transport.is_available(worker).unwrap_or(false)
                        })
                        .cloned()
                        .ok_or(DomainError::ReplicaRecoveryFailed)?;
                    self.deliver_and_execute(
                        model,
                        configuration,
                        index,
                        &source,
                        &destination,
                        frame,
                        remaining_deadline,
                        cancellation,
                    )?
                }
                Err(error) => return Err(error),
            };
            source = destination;
        }

        if frame.envelope.target
            != configuration
                .route
                .assignments
                .last()
                .ok_or(DomainError::ShardPlanInvalid)?
                .target
        {
            return Err(DomainError::FrameInvalid);
        }
        Ok(frame)
    }

    #[allow(clippy::too_many_arguments)]
    fn deliver_and_execute(
        &self,
        model: &VerifiedModel,
        configuration: &SessionConfiguration,
        index: usize,
        source: &WorkerId,
        destination: &WorkerId,
        frame: DecodedFrame,
        remaining_deadline: RemainingDeadline,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<DecodedFrame> {
        let received = self.deliver(source, destination, &frame)?;
        let assignment = &configuration.route.assignments[index];
        let plan = model
            .manifest
            .execution_plan
            .as_ref()
            .ok_or(DomainError::ShardPlanInvalid)?;
        let execution = ShardExecutionRequest {
            target: assignment.target.clone(),
            next_target: configuration
                .route
                .assignments
                .get(index + 1)
                .map(|next| next.target.clone()),
            strategy: ExecutionStrategy::layer_range(),
            requirements: ShardExecutionRequirements::LayerRange {
                shard: plan.shards[index].clone(),
            },
            input: received,
            remaining_deadline,
        };
        let backend = self
            .workers
            .get(destination)
            .ok_or(DomainError::WorkerUnavailable)?;
        let output = backend.execute(model, &execution, cancellation)?;
        output.validate_for(&execution)?;
        match output {
            ShardExecutionOutput::Boundary(frame) => {
                if index + 1 == configuration.route.assignments.len() {
                    return Err(DomainError::FrameInvalid);
                }
                self.sessions.record_checkpoint(
                    &configuration.idempotency_key,
                    frame.envelope.checkpoint_ref(),
                )?;
                Ok(frame)
            }
            ShardExecutionOutput::FinalLogits(frame)
                if index + 1 == configuration.route.assignments.len() =>
            {
                Ok(frame)
            }
            ShardExecutionOutput::FinalLogits(_) => Err(DomainError::FrameInvalid),
        }
    }

    fn deliver(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        frame: &DecodedFrame,
    ) -> DomainResult<DecodedFrame> {
        if !self.transport.is_available(destination)? {
            return Err(DomainError::WorkerUnavailable);
        }
        let bytes = FrameCodec::encode_with_extensions(
            &frame.envelope,
            &frame.payload,
            frame.compression,
            frame.trace_id.as_ref(),
            frame.extensions(),
        )?;
        self.transport.send(source, destination, bytes)?;
        let received = self
            .transport
            .receive(destination)?
            .ok_or(DomainError::WorkerUnavailable)?;
        let decoded = FrameCodec::decode(&received.bytes)?;
        self.transport.acknowledge(
            destination,
            &received.source,
            &TransportReceipt::from_envelope(&decoded.envelope),
        )?;
        Ok(decoded)
    }

    fn release_session(&self, model: &VerifiedModel, session_id: &SessionId) -> DomainResult<()> {
        for backend in self.workers.values() {
            backend.release_session(model, session_id)?;
        }
        Ok(())
    }
}

impl ShardedGenerationRuntime for LayerRangeShardedGenerationRuntime {
    fn generate(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        self.execute(model, request, cancellation, tokens)
    }
}

fn remaining_deadline(started: Instant, budget: Duration) -> DomainResult<RemainingDeadline> {
    RemainingDeadline::new(budget.saturating_sub(started.elapsed()))
}

fn sample_token(
    payload: &[u8],
    policy: &synapseflow_domain::GenerationPolicy,
    state: &mut u64,
) -> DomainResult<u32> {
    if !payload.len().is_multiple_of(4) {
        return Err(DomainError::FrameInvalid);
    }
    let logits = payload
        .chunks_exact(4)
        .map(|bytes| {
            let bytes: [u8; 4] = bytes.try_into().map_err(|_| DomainError::FrameInvalid)?;
            Ok(f32::from_le_bytes(bytes))
        })
        .collect::<DomainResult<Vec<_>>>()?;
    let maximum = logits
        .iter()
        .copied()
        .reduce(f32::max)
        .ok_or(DomainError::GenerationFailed)?;
    if !maximum.is_finite() || logits.iter().any(|value| !value.is_finite()) {
        return Err(DomainError::GenerationFailed);
    }
    let mut candidates = logits
        .iter()
        .enumerate()
        .map(|(index, logit)| (index, ((*logit - maximum) / policy.temperature).exp()))
        .collect::<Vec<_>>();
    let total = candidates
        .iter()
        .map(|(_, probability)| probability)
        .sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Err(DomainError::GenerationFailed);
    }
    for (_, probability) in &mut candidates {
        *probability /= total;
    }
    candidates.sort_by(|left, right| right.1.total_cmp(&left.1));
    let mut retained = 0.0;
    let cutoff = candidates
        .iter()
        .position(|(_, probability)| {
            retained += *probability;
            retained >= policy.top_p
        })
        .unwrap_or(candidates.len() - 1);
    candidates.truncate(cutoff + 1);
    let retained_total = candidates
        .iter()
        .map(|(_, probability)| probability)
        .sum::<f32>();
    let choice = next_unit(state) * retained_total;
    let mut cumulative = 0.0;
    let index = candidates
        .iter()
        .find_map(|(index, probability)| {
            cumulative += *probability;
            (choice <= cumulative).then_some(*index)
        })
        .unwrap_or(candidates.last().ok_or(DomainError::GenerationFailed)?.0);
    u32::try_from(index).map_err(|_| DomainError::GenerationFailed)
}

fn next_unit(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    ((*state >> 40) as f32) / ((1_u32 << 24) as f32)
}
