use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use candle_core::quantized::QMatMul;
use candle_core::{IndexOp, Module, Tensor};
use candle_nn::RmsNorm;
use synapseflow_domain::execution::{LayerRange, SessionId};
use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::ExecutionCancellation;

use crate::input::StageInput;
use crate::runtime::{LoomExecutionOutput, LoomExecutionRequest};

use super::archive::{LlamaLayout, LoomArchive};
use super::tensor::{causal_mask, repeat_key_values, rope_frequencies, MAX_CONTEXT_TOKENS};

const MAX_ACTIVE_SESSIONS: usize = 16;

pub(crate) struct LoomModel {
    range: LayerRange,
    layout: LlamaLayout,
    embeddings: Option<QMatMul>,
    layers: Vec<LoomLayer>,
    output_norm: Option<RmsNorm>,
    output_head: Option<QMatMul>,
    active_sessions: HashSet<String>,
}

impl LoomModel {
    pub(crate) fn load(artifact: &Path, range: LayerRange) -> DomainResult<Self> {
        let mut archive = LoomArchive::open(artifact)?;
        let layout = archive.layout().clone();
        if range.end_exclusive() > layout.block_count {
            return Err(DomainError::BackendIncompatible);
        }
        let (cos, sin) = rope_frequencies(
            layout.rope_dimension as usize,
            usize::try_from(layout.context_limit)
                .map_err(|_| DomainError::BackendIncompatible)?
                .min(MAX_CONTEXT_TOKENS),
            layout.rope_frequency_base,
        )
        .map_err(|_| DomainError::BackendIncompatible)?;
        let embeddings = (range.start() == 0)
            .then(|| archive.quantized_matrix("token_embd.weight"))
            .transpose()?
            .map(|weights| {
                QMatMul::from_qtensor(weights).map_err(|_| DomainError::BackendIncompatible)
            })
            .transpose()?;
        let mut layers = Vec::with_capacity((range.end_exclusive() - range.start()) as usize);
        for layer_index in range.start()..range.end_exclusive() {
            layers.push(LoomLayer::load(
                &mut archive,
                layer_index,
                &layout,
                &cos,
                &sin,
            )?);
        }
        let (output_norm, output_head) = if range.end_exclusive() == layout.block_count {
            let norm = RmsNorm::new(
                archive.norm("output_norm.weight")?,
                f64::from(layout.rms_epsilon),
            );
            let head = QMatMul::from_qtensor(archive.quantized_matrix("output.weight")?)
                .map_err(|_| DomainError::BackendIncompatible)?;
            (Some(norm), Some(head))
        } else {
            (None, None)
        };
        Ok(Self {
            range,
            layout,
            embeddings,
            layers,
            output_norm,
            output_head,
            active_sessions: HashSet::new(),
        })
    }

    pub(crate) fn execute(
        &mut self,
        request: &LoomExecutionRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<LoomExecutionOutput> {
        if request.declared_range != self.range {
            return Err(DomainError::ShardPlanInvalid);
        }
        let session = request.session_id.as_str().to_owned();
        let (mut activations, token_count, position_start) = self.input_tensor(&request.input)?;
        let started = Instant::now();
        let position_start =
            usize::try_from(position_start).map_err(|_| DomainError::FrameBoundsExceeded)?;
        let context_end = position_start
            .checked_add(token_count)
            .ok_or(DomainError::FrameBoundsExceeded)?;
        if context_end > self.context_limit() {
            return Err(DomainError::FrameBoundsExceeded);
        }
        self.prepare_session(&session, position_start)?;
        for layer in &mut self.layers {
            if cancellation.is_cancelled() {
                return Err(DomainError::SessionCancelled);
            }
            if started.elapsed() >= request.remaining_deadline.duration() {
                return Err(DomainError::DeadlineExceeded);
            }
            activations = layer.forward(&activations, position_start, &session)?;
        }
        if started.elapsed() >= request.remaining_deadline.duration() {
            return Err(DomainError::DeadlineExceeded);
        }
        if !request.final_stage {
            return activations
                .flatten_all()
                .and_then(|values| values.to_vec1::<f32>())
                .map(|activations| LoomExecutionOutput::Boundary {
                    activations,
                    token_count,
                })
                .map_err(|_| DomainError::GenerationFailed);
        }
        let norm = self
            .output_norm
            .as_ref()
            .ok_or(DomainError::BackendIncompatible)?;
        let head = self
            .output_head
            .as_ref()
            .ok_or(DomainError::BackendIncompatible)?;
        let logits = norm
            .forward(&activations)
            .and_then(|values| values.i((.., token_count - 1, ..)))
            .and_then(|values| head.forward(&values))
            .and_then(|values| values.flatten_all())
            .and_then(|values| values.to_vec1::<f32>())
            .map_err(|_| DomainError::GenerationFailed)?;
        if logits.len() != self.layout.vocabulary_size as usize {
            return Err(DomainError::GenerationFailed);
        }
        Ok(LoomExecutionOutput::FinalLogits(logits))
    }

    pub(crate) fn discard_session(&mut self, session: &SessionId) {
        let key = session.as_str();
        self.active_sessions.remove(key);
        for layer in &mut self.layers {
            layer.discard_session(key);
        }
    }

    fn input_tensor(&self, input: &StageInput) -> DomainResult<(Tensor, usize, u64)> {
        match input {
            StageInput::TokenIds {
                token_ids,
                position_start,
            } => {
                let embeddings = self
                    .embeddings
                    .as_ref()
                    .ok_or(DomainError::FrameDtypeUnsupported)?;
                let token_count = token_ids.len();
                Tensor::new(token_ids.as_slice(), &candle_core::Device::Cpu)
                    .and_then(|ids| ids.reshape((1, token_count)))
                    .and_then(|ids| embeddings.embedding(&ids))
                    .map(|values| (values, token_count, *position_start))
                    .map_err(|_| DomainError::GenerationFailed)
            }
            StageInput::Boundary {
                activations,
                token_count,
                position_start,
            } => Tensor::from_vec(
                activations.clone(),
                (1, *token_count, self.layout.embedding_width as usize),
                &candle_core::Device::Cpu,
            )
            .map(|values| (values, *token_count, *position_start))
            .map_err(|_| DomainError::FrameInvalid),
        }
    }

    fn prepare_session(&mut self, session: &str, position_start: usize) -> DomainResult<()> {
        if position_start == 0 {
            if !self.active_sessions.contains(session)
                && self.active_sessions.len() >= MAX_ACTIVE_SESSIONS
            {
                return Err(DomainError::BackendUnavailable);
            }
            self.active_sessions.insert(session.to_owned());
            for layer in &mut self.layers {
                layer.discard_session(session);
            }
            return Ok(());
        }
        if self.active_sessions.contains(session) {
            return Ok(());
        }
        Err(DomainError::SessionStateInvalid)
    }

    fn context_limit(&self) -> usize {
        (self.layout.context_limit as usize).min(MAX_CONTEXT_TOKENS)
    }
}

struct LoomLayer {
    attention_q: QMatMul,
    attention_k: QMatMul,
    attention_v: QMatMul,
    attention_output: QMatMul,
    attention_norm: RmsNorm,
    feed_forward_gate: QMatMul,
    feed_forward_down: QMatMul,
    feed_forward_up: QMatMul,
    feed_forward_norm: RmsNorm,
    attention_heads: usize,
    key_value_heads: usize,
    head_dimension: usize,
    cos: Tensor,
    sin: Tensor,
    negative_infinity: Tensor,
    caches: HashMap<String, LayerCache>,
}

impl LoomLayer {
    fn load(
        archive: &mut LoomArchive,
        index: u32,
        layout: &LlamaLayout,
        cos: &Tensor,
        sin: &Tensor,
    ) -> DomainResult<Self> {
        let prefix = format!("blk.{index}");
        Ok(Self {
            attention_q: qmatmul(archive, &format!("{prefix}.attn_q.weight"))?,
            attention_k: qmatmul(archive, &format!("{prefix}.attn_k.weight"))?,
            attention_v: qmatmul(archive, &format!("{prefix}.attn_v.weight"))?,
            attention_output: qmatmul(archive, &format!("{prefix}.attn_output.weight"))?,
            attention_norm: RmsNorm::new(
                archive.norm(&format!("{prefix}.attn_norm.weight"))?,
                f64::from(layout.rms_epsilon),
            ),
            feed_forward_gate: qmatmul(archive, &format!("{prefix}.ffn_gate.weight"))?,
            feed_forward_down: qmatmul(archive, &format!("{prefix}.ffn_down.weight"))?,
            feed_forward_up: qmatmul(archive, &format!("{prefix}.ffn_up.weight"))?,
            feed_forward_norm: RmsNorm::new(
                archive.norm(&format!("{prefix}.ffn_norm.weight"))?,
                f64::from(layout.rms_epsilon),
            ),
            attention_heads: layout.attention_heads as usize,
            key_value_heads: layout.key_value_heads as usize,
            head_dimension: (layout.embedding_width / layout.attention_heads) as usize,
            cos: cos.clone(),
            sin: sin.clone(),
            negative_infinity: Tensor::new(f32::NEG_INFINITY, &candle_core::Device::Cpu)
                .map_err(|_| DomainError::BackendIncompatible)?,
            caches: HashMap::new(),
        })
    }

    fn forward(
        &mut self,
        activations: &Tensor,
        position_start: usize,
        session: &str,
    ) -> DomainResult<Tensor> {
        let residual = activations;
        let normalized = self
            .attention_norm
            .forward(activations)
            .map_err(|_| DomainError::GenerationFailed)?;
        let attention = self.attention(&normalized, position_start, session)?;
        let activations = (attention + residual).map_err(|_| DomainError::GenerationFailed)?;
        let residual = &activations;
        let normalized = self
            .feed_forward_norm
            .forward(&activations)
            .map_err(|_| DomainError::GenerationFailed)?;
        let gate = self
            .feed_forward_gate
            .forward(&normalized)
            .map_err(|_| DomainError::GenerationFailed)?;
        let up = self
            .feed_forward_up
            .forward(&normalized)
            .map_err(|_| DomainError::GenerationFailed)?;
        let feed_forward = candle_nn::ops::silu(&gate)
            .and_then(|values| values.broadcast_mul(&up))
            .and_then(|values| self.feed_forward_down.forward(&values))
            .map_err(|_| DomainError::GenerationFailed)?;
        (feed_forward + residual).map_err(|_| DomainError::GenerationFailed)
    }

    fn discard_session(&mut self, session: &str) {
        self.caches.remove(session);
    }

    fn attention(
        &mut self,
        activations: &Tensor,
        position_start: usize,
        session: &str,
    ) -> DomainResult<Tensor> {
        let (batch, token_count, embedding_width) = activations
            .dims3()
            .map_err(|_| DomainError::GenerationFailed)?;
        let q = self
            .attention_q
            .forward(activations)
            .and_then(|values| {
                values.reshape((
                    batch,
                    token_count,
                    self.attention_heads,
                    self.head_dimension,
                ))
            })
            .and_then(|values| values.transpose(1, 2))
            .map_err(|_| DomainError::GenerationFailed)?;
        let k = self
            .attention_k
            .forward(activations)
            .and_then(|values| {
                values.reshape((
                    batch,
                    token_count,
                    self.key_value_heads,
                    self.head_dimension,
                ))
            })
            .and_then(|values| values.transpose(1, 2))
            .map_err(|_| DomainError::GenerationFailed)?;
        let v = self
            .attention_v
            .forward(activations)
            .and_then(|values| {
                values.reshape((
                    batch,
                    token_count,
                    self.key_value_heads,
                    self.head_dimension,
                ))
            })
            .and_then(|values| values.transpose(1, 2))
            .and_then(|values| values.contiguous())
            .map_err(|_| DomainError::GenerationFailed)?;
        let q = self.rotary(&q, position_start)?;
        let k = self.rotary(&k, position_start)?;
        let cache = self.caches.entry(session.to_owned()).or_default();
        let (keys, values) = cache.append(k, v, position_start)?;
        let keys = repeat_key_values(keys, self.attention_heads / self.key_value_heads)
            .map_err(|_| DomainError::GenerationFailed)?;
        let values = repeat_key_values(values, self.attention_heads / self.key_value_heads)
            .map_err(|_| DomainError::GenerationFailed)?;
        let mask =
            causal_mask(token_count, position_start).map_err(|_| DomainError::GenerationFailed)?;
        let attention = (q
            .matmul(&keys.t().map_err(|_| DomainError::GenerationFailed)?)
            .map_err(|_| DomainError::GenerationFailed)?
            / (self.head_dimension as f64).sqrt())
        .map_err(|_| DomainError::GenerationFailed)?;
        let mask = mask
            .broadcast_as(attention.shape())
            .map_err(|_| DomainError::GenerationFailed)?;
        let key_values = values
            .contiguous()
            .map_err(|_| DomainError::GenerationFailed)?;
        let attention = mask
            .where_cond(
                &self
                    .negative_infinity
                    .broadcast_as(attention.shape())
                    .map_err(|_| DomainError::GenerationFailed)?,
                &attention,
            )
            .and_then(|values| candle_nn::ops::softmax_last_dim(&values))
            .and_then(|weights| weights.matmul(&key_values))
            .map_err(|_| DomainError::GenerationFailed)?;
        let attention = attention
            .transpose(1, 2)
            .and_then(|values| values.reshape((batch, token_count, embedding_width)))
            .and_then(|values| self.attention_output.forward(&values))
            .map_err(|_| DomainError::GenerationFailed)?;
        Ok(attention)
    }

    fn rotary(&self, values: &Tensor, position_start: usize) -> DomainResult<Tensor> {
        let (_, _, token_count, _) = values.dims4().map_err(|_| DomainError::GenerationFailed)?;
        let cos = self
            .cos
            .narrow(0, position_start, token_count)
            .map_err(|_| DomainError::FrameBoundsExceeded)?;
        let sin = self
            .sin
            .narrow(0, position_start, token_count)
            .map_err(|_| DomainError::FrameBoundsExceeded)?;
        candle_nn::rotary_emb::rope_i(
            &values
                .contiguous()
                .map_err(|_| DomainError::GenerationFailed)?,
            &cos,
            &sin,
        )
        .map_err(|_| DomainError::GenerationFailed)
    }
}

#[derive(Default)]
struct LayerCache {
    keys: Option<Tensor>,
    values: Option<Tensor>,
}

impl LayerCache {
    fn append(
        &mut self,
        keys: Tensor,
        values: Tensor,
        position_start: usize,
    ) -> DomainResult<(Tensor, Tensor)> {
        let (keys, values) = match (&self.keys, &self.values) {
            (Some(previous_keys), Some(previous_values)) if position_start > 0 => (
                Tensor::cat(&[previous_keys, &keys], 2),
                Tensor::cat(&[previous_values, &values], 2),
            ),
            (None, None) if position_start == 0 => (Ok(keys), Ok(values)),
            _ => return Err(DomainError::SessionStateInvalid),
        };
        let keys = keys.map_err(|_| DomainError::GenerationFailed)?;
        let values = values.map_err(|_| DomainError::GenerationFailed)?;
        self.keys = Some(keys.clone());
        self.values = Some(values.clone());
        Ok((keys, values))
    }
}

fn qmatmul(archive: &mut LoomArchive, name: &str) -> DomainResult<QMatMul> {
    QMatMul::from_qtensor(archive.quantized_matrix(name)?)
        .map_err(|_| DomainError::BackendIncompatible)
}
