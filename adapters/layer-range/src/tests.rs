use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use candle_core::quantized::gguf_file::{self, Value};
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{DType, Device, Tensor};

use synapseflow_domain::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
    RemainingDeadline, SessionId, StreamId, TensorDescriptor, TensorDtype,
};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DecodedFrame, DomainError, DomainResult, ExecutionStrategy,
    FrameCodec, FrameCompression, FrameExtension, LayerRange, ModelFormat, ModelManifest,
    ModelReference, ShardId, ShardPlan, ShardSpec, TokenizerDeclaration, TokenizerKind,
    LOOM_RUNTIME_PROFILE,
};
use synapseflow_ports::{
    ExecutionCancellation, NeverCancelled, ShardExecutionBackend, ShardExecutionOutput,
    ShardExecutionRequest, ShardExecutionRequirements, VerifiedModel,
};

use crate::{
    LoomBackend, LoomExecutionOutput, LoomExecutionRequest, LoomExecutor, LoomModelLayout,
};

struct RecordingRuntime {
    calls: Mutex<Vec<(String, LoomExecutionRequest)>>,
    invalid_first_output: bool,
}

impl RecordingRuntime {
    fn valid() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            invalid_first_output: false,
        }
    }
}

impl LoomExecutor for RecordingRuntime {
    fn inspect(&self, _: &Path) -> DomainResult<LoomModelLayout> {
        Ok(LoomModelLayout {
            layer_count: 2,
            activation_width: 3,
            vocabulary_size: 5,
        })
    }

    fn execute(
        &self,
        artifact: &Path,
        request: &LoomExecutionRequest,
        _: &dyn ExecutionCancellation,
    ) -> DomainResult<LoomExecutionOutput> {
        self.calls
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .push((artifact.display().to_string(), request.clone()));
        if request.final_stage || self.invalid_first_output {
            Ok(LoomExecutionOutput::FinalLogits(vec![0.5; 5]))
        } else {
            let token_count = match &request.input {
                crate::StageInput::TokenIds { token_ids, .. } => token_ids.len(),
                crate::StageInput::Boundary { token_count, .. } => *token_count,
            };
            Ok(LoomExecutionOutput::Boundary {
                activations: vec![0.25; token_count * 3],
                token_count,
            })
        }
    }
}

struct Cancelled;

impl ExecutionCancellation for Cancelled {
    fn is_cancelled(&self) -> bool {
        true
    }
}

fn model() -> VerifiedModel {
    model_with_paths(vec![
        "C:/verified/first.gguf".into(),
        "C:/verified/second.gguf".into(),
    ])
}

fn model_with_paths(artifact_paths: Vec<PathBuf>) -> VerifiedModel {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/layer-range@sha256:{}",
        "a".repeat(64)
    ))
    .expect("fixture reference is valid");
    let first = shard("first", "first-weights", 0, 1);
    let second = shard("second", "second-weights", 1, 2);
    let artifacts = vec![
        artifact("first-weights", "b"),
        artifact("second-weights", "c"),
    ];
    VerifiedModel::with_cached_artifacts(
        ModelManifest {
            reference,
            schema_version: 2,
            model_id: "generated-layer-range-fixture".to_owned(),
            model_version: "v1".to_owned(),
            format: ModelFormat::Gguf,
            architecture: "llama".to_owned(),
            quantization: "Q5_K_M".to_owned(),
            tokenizer: TokenizerDeclaration {
                kind: TokenizerKind::Embedded,
                model: "llama".to_owned(),
            },
            artifacts,
            publisher_key_id: "ed25519:fixture".to_owned(),
            license: "MIT".to_owned(),
            provenance: "generated:layer-range".to_owned(),
            execution_plan: Some(
                ShardPlan::new(ExecutionStrategy::layer_range(), vec![first, second])
                    .expect("fixture plan is valid"),
            ),
            runtime_profile: Some(LOOM_RUNTIME_PROFILE.to_owned()),
        },
        artifact_paths,
    )
    .expect("verified fixture paths bind to declared artifacts")
}

fn generated_gguf_fixture(layer: u32, includes_embeddings: bool, includes_output: bool) -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("synapseflow-loom-{layer}-{suffix}.gguf"));
    let mut file = File::create(&path).expect("generated GGUF should be writable");
    let matrix = || {
        QTensor::quantize(
            &Tensor::zeros((256, 256), DType::F32, &Device::Cpu)
                .expect("zero matrix should allocate"),
            GgmlDType::Q5K,
        )
        .expect("zero matrix should quantize")
    };
    let norm = || {
        QTensor::quantize(
            &Tensor::zeros(256, DType::F32, &Device::Cpu).expect("zero norm should allocate"),
            GgmlDType::F32,
        )
        .expect("zero norm should quantize")
    };
    let mut tensors = Vec::new();
    if includes_embeddings {
        tensors.push(("token_embd.weight".to_owned(), matrix()));
    }
    if includes_output {
        tensors.push(("output_norm.weight".to_owned(), norm()));
        tensors.push(("output.weight".to_owned(), matrix()));
    }
    for suffix in [
        "attn_q.weight",
        "attn_k.weight",
        "attn_v.weight",
        "attn_output.weight",
        "ffn_gate.weight",
        "ffn_down.weight",
        "ffn_up.weight",
    ] {
        tensors.push((format!("blk.{layer}.{suffix}"), matrix()));
    }
    tensors.push((format!("blk.{layer}.attn_norm.weight"), norm()));
    tensors.push((format!("blk.{layer}.ffn_norm.weight"), norm()));
    let tokenizer_tokens = (0..256)
        .map(|token| Value::String(format!("token-{token}")))
        .collect::<Vec<_>>();
    let metadata = vec![
        ("general.architecture", Value::String("llama".to_owned())),
        ("llama.block_count", Value::U32(2)),
        ("llama.embedding_length", Value::U32(256)),
        ("llama.vocab_size", Value::U32(256)),
        ("llama.attention.head_count", Value::U32(1)),
        ("llama.attention.head_count_kv", Value::U32(1)),
        ("llama.rope.dimension_count", Value::U32(256)),
        ("llama.context_length", Value::U32(8)),
        ("llama.attention.layer_norm_rms_epsilon", Value::F32(1e-5)),
        ("llama.rope.freq_base", Value::F32(10_000.0)),
        ("tokenizer.ggml.tokens", Value::Array(tokenizer_tokens)),
    ];
    let tensor_refs = tensors
        .iter()
        .map(|(name, tensor)| (name.as_str(), tensor))
        .collect::<Vec<_>>();
    let metadata_refs = metadata
        .iter()
        .map(|(name, value)| (*name, value))
        .collect::<Vec<_>>();
    gguf_file::write(&mut file, &metadata_refs, &tensor_refs)
        .expect("generated GGUF should encode");
    path
}

fn artifact(id: &str, hash_byte: &str) -> ArtifactDescriptor {
    ArtifactDescriptor {
        id: ArtifactId::new(id.to_owned()).expect("fixture artifact is valid"),
        uri: format!("https://fixtures.example/{id}.gguf"),
        content_sha256: format!("sha256:{}", hash_byte.repeat(64)),
        size_bytes: 1,
    }
}

fn shard(id: &str, artifact_id: &str, start: u32, end: u32) -> ShardSpec {
    ShardSpec::new(
        ShardId::new(id.to_owned()).expect("fixture shard ID is valid"),
        ArtifactId::new(artifact_id.to_owned()).expect("fixture artifact ID is valid"),
        LayerRange::new(start, end).expect("fixture range is valid"),
        1,
    )
    .expect("fixture shard is valid")
}

fn target(model: &VerifiedModel, shard: &str) -> FrameTarget {
    FrameTarget {
        model: model.manifest.reference.clone(),
        shard: ShardId::new(shard.to_owned()).expect("fixture shard ID is valid"),
    }
}

fn frame(
    target: FrameTarget,
    dtype: TensorDtype,
    dimensions: Vec<u32>,
    payload: Vec<u8>,
) -> DecodedFrame {
    let envelope = FrameEnvelope::new(
        FrameProtocolVersion::current(),
        FrameMessageType::Data,
        SessionId::new("session-00000001".to_owned()).expect("fixture session is valid"),
        StreamId::new(1).expect("fixture stream is valid"),
        FrameSequence::initial(),
        target,
        Some(TensorDescriptor::new(dtype, dimensions).expect("fixture tensor is valid")),
        RemainingDeadline::new(Duration::from_millis(100)).expect("fixture deadline is valid"),
    )
    .expect("fixture envelope is valid");
    FrameCodec::decode(
        &FrameCodec::encode_with_extensions(
            &envelope,
            &payload,
            FrameCompression::None,
            None,
            &[FrameExtension::new(2, 0_u64.to_be_bytes().to_vec())
                .expect("position extension is valid")],
        )
        .expect("fixture frame encodes"),
    )
    .expect("fixture frame decodes")
}

fn request(
    model: &VerifiedModel,
    shard_index: usize,
    input: DecodedFrame,
) -> ShardExecutionRequest {
    let plan = model
        .manifest
        .execution_plan
        .as_ref()
        .expect("fixture plan exists");
    ShardExecutionRequest {
        target: target(model, plan.shards[shard_index].id().as_str()),
        next_target: plan
            .shards
            .get(shard_index + 1)
            .map(|next| target(model, next.id().as_str())),
        strategy: ExecutionStrategy::layer_range(),
        requirements: ShardExecutionRequirements::LayerRange {
            shard: plan.shards[shard_index].clone(),
        },
        input,
        remaining_deadline: RemainingDeadline::new(Duration::from_millis(100))
            .expect("fixture deadline is valid"),
    }
}

#[test]
fn executes_only_declared_ranges_and_transfers_a_codec_validated_boundary() {
    let model = model();
    let runtime = Arc::new(RecordingRuntime::valid());
    let backend = LoomBackend::with_executor(runtime.clone());
    let first = request(
        &model,
        0,
        frame(
            target(&model, "first"),
            TensorDtype::U32,
            vec![2],
            [11_u32, 12]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect(),
        ),
    );

    let boundary = backend
        .execute(&model, &first, &NeverCancelled)
        .expect("first range should execute");
    let ShardExecutionOutput::Boundary(boundary) = boundary else {
        panic!("first range must produce a boundary");
    };
    assert_eq!(boundary.envelope.target, target(&model, "second"));
    assert_eq!(boundary.envelope.sequence.value(), 1);
    assert_eq!(
        boundary
            .envelope
            .tensor
            .as_ref()
            .expect("tensor exists")
            .dtype,
        TensorDtype::F32
    );
    assert_eq!(
        boundary
            .envelope
            .tensor
            .as_ref()
            .expect("tensor exists")
            .dimensions
            .as_slice(),
        &[2, 3]
    );

    let final_request = request(&model, 1, boundary);
    let final_output = backend
        .execute(&model, &final_request, &NeverCancelled)
        .expect("final range should execute");
    let ShardExecutionOutput::FinalLogits(logits) = final_output else {
        panic!("last range must produce logits");
    };
    assert_eq!(logits.envelope.target, target(&model, "second"));
    assert_eq!(logits.payload.len(), 20);

    let calls = runtime.calls.lock().expect("calls are readable");
    assert_eq!(calls.len(), 2);
    assert!(calls[0].0.ends_with("first.gguf"));
    assert_eq!(
        calls[0].1.declared_range,
        LayerRange::new(0, 1).expect("range is valid")
    );
    assert!(!calls[0].1.final_stage);
    assert!(calls[1].0.ends_with("second.gguf"));
    assert_eq!(
        calls[1].1.declared_range,
        LayerRange::new(1, 2).expect("range is valid")
    );
    assert!(calls[1].1.final_stage);
}

#[test]
fn rejects_wrong_stage_dtype_invalid_loom_output_and_cancellation() {
    let model = model();
    let valid_input = frame(
        target(&model, "first"),
        TensorDtype::U32,
        vec![1],
        7_u32.to_le_bytes().to_vec(),
    );
    let execution_request = request(&model, 0, valid_input.clone());
    let backend = LoomBackend::with_executor(Arc::new(RecordingRuntime {
        invalid_first_output: true,
        ..RecordingRuntime::valid()
    }));
    assert!(matches!(
        backend.execute(&model, &execution_request, &NeverCancelled),
        Err(DomainError::GenerationFailed)
    ));
    assert!(matches!(
        backend.execute(&model, &execution_request, &Cancelled),
        Err(DomainError::SessionCancelled)
    ));

    let wrong_dtype = request(
        &model,
        0,
        frame(
            target(&model, "first"),
            TensorDtype::F32,
            vec![1, 1],
            0.0_f32.to_le_bytes().to_vec(),
        ),
    );
    assert!(matches!(
        backend.execute(&model, &wrong_dtype, &NeverCancelled),
        Err(DomainError::FrameDtypeUnsupported)
    ));
}

#[test]
fn production_loom_engine_executes_two_declared_ranges_from_a_generated_gguf() {
    let first_artifact = generated_gguf_fixture(0, true, false);
    let final_artifact = generated_gguf_fixture(1, false, true);
    let model = model_with_paths(vec![first_artifact.clone(), final_artifact.clone()]);
    let first = request(
        &model,
        0,
        frame(
            target(&model, "first"),
            TensorDtype::U32,
            vec![2],
            [11_u32, 12]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect(),
        ),
    );
    let boundary = LoomBackend::new()
        .execute(&model, &first, &NeverCancelled)
        .expect("first Loom range should execute");
    let ShardExecutionOutput::Boundary(boundary) = boundary else {
        panic!("first Loom range must return a boundary");
    };
    let final_request = request(&model, 1, boundary);
    let final_output = LoomBackend::new()
        .execute(&model, &final_request, &NeverCancelled)
        .expect("final Loom range should execute");
    let ShardExecutionOutput::FinalLogits(logits) = final_output else {
        panic!("final Loom range must return logits");
    };
    assert_eq!(logits.payload.len(), 256 * 4);
    assert!(logits.payload.iter().all(|byte| *byte == 0));
    std::fs::remove_file(first_artifact).expect("first generated GGUF should be removable");
    std::fs::remove_file(final_artifact).expect("final generated GGUF should be removable");
}
