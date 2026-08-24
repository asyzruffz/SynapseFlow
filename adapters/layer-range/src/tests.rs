use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use candle_core::quantized::gguf_file::{self, Value};
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{DType, Device, Tensor};

use synapseflow_adapter_in_memory::{InMemoryAuditSink, InMemoryPeerDirectory};
use synapseflow_adapter_loopback::{LoopbackFault, LoopbackNetwork};
use synapseflow_application::{
    ExecutionRoute, IdempotencyKey, SessionConfiguration, SessionManager, ShardPlanner,
};
use synapseflow_domain::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
    InFlightFrameLimit, RemainingDeadline, RetryBudget, SessionId, StreamId, TensorDescriptor,
    TensorDtype,
};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DecodedFrame, DomainError, DomainResult, ExecutionStrategy,
    FrameCodec, FrameCompression, FrameExtension, LayerRange, ModelFormat, ModelManifest,
    ModelReference, ShardId, ShardPlan, ShardSpec, TokenizerDeclaration, TokenizerKind,
    LOOM_RUNTIME_PROFILE,
};
use synapseflow_ports::{
    AuditEvent, ExecutionCancellation, NeverCancelled, ShardAvailability, ShardExecutionBackend,
    ShardExecutionOutput, ShardExecutionRequest, ShardExecutionRequirements, ShardSessionOutcome,
    VerifiedModel, WorkerCapability, WorkerHealth, WorkerId,
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
    model_with_replica_paths(artifact_paths, 1)
}

fn model_with_replica_paths(artifact_paths: Vec<PathBuf>, minimum_replicas: u8) -> VerifiedModel {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/layer-range@sha256:{}",
        "a".repeat(64)
    ))
    .expect("fixture reference is valid");
    let first = shard_with_replicas("first", "first-weights", 0, 1, minimum_replicas);
    let second = shard_with_replicas("second", "second-weights", 1, 2, minimum_replicas);
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

fn whole_model_with_path(artifact_path: PathBuf) -> VerifiedModel {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/layer-range@sha256:{}",
        "d".repeat(64)
    ))
    .expect("fixture reference is valid");
    let whole = shard("whole", "whole-weights", 0, 2);
    VerifiedModel::with_cached_artifacts(
        ModelManifest {
            reference,
            schema_version: 2,
            model_id: "generated-layer-range-baseline".to_owned(),
            model_version: "v1".to_owned(),
            format: ModelFormat::Gguf,
            architecture: "llama".to_owned(),
            quantization: "Q5_K_M".to_owned(),
            tokenizer: TokenizerDeclaration {
                kind: TokenizerKind::Embedded,
                model: "llama".to_owned(),
            },
            artifacts: vec![artifact("whole-weights", "d")],
            publisher_key_id: "ed25519:fixture".to_owned(),
            license: "MIT".to_owned(),
            provenance: "generated:whole-model-baseline".to_owned(),
            execution_plan: Some(
                ShardPlan::new(ExecutionStrategy::layer_range(), vec![whole])
                    .expect("fixture plan is valid"),
            ),
            runtime_profile: Some(LOOM_RUNTIME_PROFILE.to_owned()),
        },
        vec![artifact_path],
    )
    .expect("verified fixture path binds to declared artifact")
}

fn generated_gguf_fixture(
    layers: &[u32],
    includes_embeddings: bool,
    includes_output: bool,
) -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let layer_label = layers
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join("-");
    let path = std::env::temp_dir().join(format!("synapseflow-loom-{layer_label}-{suffix}.gguf"));
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
        tensors.push((
            "token_embd.weight".to_owned(),
            QTensor::quantize(
                &Tensor::ones((256, 256), DType::F32, &Device::Cpu)
                    .expect("embedding matrix should allocate"),
                GgmlDType::Q5K,
            )
            .expect("embedding matrix should quantize"),
        ));
    }
    if includes_output {
        tensors.push((
            "output_norm.weight".to_owned(),
            QTensor::quantize(
                &Tensor::ones(256, DType::F32, &Device::Cpu).expect("output norm should allocate"),
                GgmlDType::F32,
            )
            .expect("output norm should quantize"),
        ));
        tensors.push((
            "output.weight".to_owned(),
            QTensor::quantize(
                &Tensor::ones((256, 256), DType::F32, &Device::Cpu)
                    .expect("output matrix should allocate"),
                GgmlDType::Q5K,
            )
            .expect("output matrix should quantize"),
        ));
    }
    for layer in layers {
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
    }
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
    shard_with_replicas(id, artifact_id, start, end, 1)
}

fn shard_with_replicas(
    id: &str,
    artifact_id: &str,
    start: u32,
    end: u32,
    minimum_replicas: u8,
) -> ShardSpec {
    ShardSpec::new(
        ShardId::new(id.to_owned()).expect("fixture shard ID is valid"),
        ArtifactId::new(artifact_id.to_owned()).expect("fixture artifact ID is valid"),
        LayerRange::new(start, end).expect("fixture range is valid"),
        minimum_replicas,
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

fn harness_directory(model: &VerifiedModel) -> Arc<InMemoryPeerDirectory> {
    let plan = model
        .manifest
        .execution_plan
        .as_ref()
        .expect("fixture plan exists");
    let availability = |index: usize| ShardAvailability {
        model: model.manifest.reference.clone(),
        shard: plan.shards[index].id().clone(),
    };
    let worker = |id: &str, shards: Vec<ShardAvailability>| {
        WorkerCapability::new(
            WorkerId::new(id.to_owned()).expect("fixture worker ID is valid"),
            WorkerHealth::Healthy,
            vec![ExecutionStrategy::layer_range()],
            shards,
        )
        .expect("fixture worker capability is valid")
    };
    Arc::new(InMemoryPeerDirectory::new(vec![
        worker("loopback-a", vec![availability(0)]),
        worker("loopback-b", vec![availability(0), availability(1)]),
        worker("loopback-c", vec![availability(1)]),
    ]))
}

fn running_session(
    route: ExecutionRoute,
) -> (SessionManager, SessionConfiguration, Arc<InMemoryAuditSink>) {
    let configuration = SessionConfiguration {
        idempotency_key: IdempotencyKey::new("loom-harness-run-0001".to_owned())
            .expect("fixture idempotency key is valid"),
        session_id: SessionId::new("session-00000001".to_owned())
            .expect("fixture session ID is valid"),
        route,
        remaining_deadline: RemainingDeadline::new(Duration::from_secs(5))
            .expect("fixture deadline is valid"),
        retry_budget: RetryBudget::new(1),
    };
    let audit = Arc::new(InMemoryAuditSink::default());
    let manager = SessionManager::new(audit.clone());
    manager
        .begin(configuration.clone())
        .expect("session should begin");
    manager
        .mark_planned(&configuration.idempotency_key)
        .expect("session should be planned");
    manager
        .start(&configuration.idempotency_key)
        .expect("session should start");
    (manager, configuration, audit)
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
    let first_artifact = generated_gguf_fixture(&[0], true, false);
    let final_artifact = generated_gguf_fixture(&[1], false, true);
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

#[test]
fn loopback_harness_matches_contiguous_loom_and_recovers_the_final_range_from_checkpoint() {
    let whole_artifact = generated_gguf_fixture(&[0, 1], true, true);
    let first_artifact = generated_gguf_fixture(&[0], true, false);
    let final_artifact = generated_gguf_fixture(&[1], false, true);
    let whole_model = whole_model_with_path(whole_artifact.clone());
    let sharded_model =
        model_with_replica_paths(vec![first_artifact.clone(), final_artifact.clone()], 2);
    let token_payload = [11_u32, 12]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();

    let baseline = LoomBackend::new()
        .execute(
            &whole_model,
            &request(
                &whole_model,
                0,
                frame(
                    target(&whole_model, "whole"),
                    TensorDtype::U32,
                    vec![2],
                    token_payload.clone(),
                ),
            ),
            &NeverCancelled,
        )
        .expect("whole model should execute contiguously");
    let ShardExecutionOutput::FinalLogits(baseline) = baseline else {
        panic!("whole model should return final logits");
    };
    assert_eq!(baseline.payload.len(), 256 * 4);

    let route = ShardPlanner::new(harness_directory(&sharded_model))
        .plan(&sharded_model.manifest)
        .expect("two-range manifest should have a deterministic route");
    assert_eq!(route.assignments[0].primary.as_str(), "loopback-a");
    assert_eq!(route.assignments[1].primary.as_str(), "loopback-b");
    assert_eq!(route.assignments[1].replicas[1].as_str(), "loopback-c");
    let (sessions, configuration, audit) = running_session(route.clone());

    let network = LoopbackNetwork::new(
        InFlightFrameLimit::new(4).expect("fixture queue limit is valid"),
        vec![
            route.assignments[0].primary.clone(),
            route.assignments[1].primary.clone(),
            route.assignments[1].replicas[1].clone(),
        ],
    )
    .expect("workers should form a loopback network");
    let first_worker = network
        .worker(&route.assignments[0].primary)
        .expect("first worker should exist");
    let primary_final_worker = network
        .worker(&route.assignments[1].primary)
        .expect("primary final worker should exist");
    let replica_final_worker = network
        .worker(&route.assignments[1].replicas[1])
        .expect("replica final worker should exist");

    let initial = frame(
        target(&sharded_model, "first"),
        TensorDtype::U32,
        vec![2],
        token_payload,
    );
    first_worker
        .send_frame(first_worker.id(), &initial)
        .expect("initial frame should cross the worker codec boundary");
    let received_initial = first_worker
        .receive()
        .expect("first worker should receive input")
        .expect("initial frame should arrive")
        .frame;
    let first_output = LoomBackend::new()
        .execute(
            &sharded_model,
            &request(&sharded_model, 0, received_initial),
            &NeverCancelled,
        )
        .expect("first range should execute");
    let ShardExecutionOutput::Boundary(boundary) = first_output else {
        panic!("first range should return an activation boundary");
    };
    sessions
        .record_checkpoint(
            &configuration.idempotency_key,
            boundary.envelope.checkpoint_ref(),
        )
        .expect("boundary should become the recovery checkpoint");

    network
        .inject(LoopbackFault::Unavailable {
            worker: route.assignments[1].primary.clone(),
            enabled: true,
        })
        .expect("primary worker should become unavailable");
    assert!(matches!(
        first_worker.send_frame(&route.assignments[1].primary, &boundary),
        Err(DomainError::WorkerUnavailable)
    ));
    let recovery = sessions
        .retry_from_latest_checkpoint(&configuration.idempotency_key, true)
        .expect("session should select its latest checkpoint for one recovery attempt");
    assert_eq!(recovery.checkpoint, boundary.envelope.checkpoint_ref());

    first_worker
        .send_frame(replica_final_worker.id(), &boundary)
        .expect("checkpoint boundary should reach the final-range replica");
    let replayed_boundary = replica_final_worker
        .receive()
        .expect("replica should receive the checkpoint boundary")
        .expect("checkpoint boundary should arrive")
        .frame;
    let recovered_output = LoomBackend::new()
        .execute(
            &sharded_model,
            &request(&sharded_model, 1, replayed_boundary),
            &NeverCancelled,
        )
        .expect("replica should execute the final range from checkpoint");
    let ShardExecutionOutput::FinalLogits(recovered_output) = recovered_output else {
        panic!("final range should return logits");
    };
    assert_eq!(recovered_output.payload, baseline.payload);

    primary_final_worker
        .shutdown()
        .expect("unavailable primary should shut down idempotently");
    let completed = sessions
        .complete(&configuration.idempotency_key)
        .expect("recovered session should complete");
    assert_eq!(completed.fallback_count, 1);
    assert_eq!(completed.retry_count, 1);
    assert!(audit
        .events()
        .expect("audit should be readable")
        .iter()
        .all(|event| matches!(
            event,
            AuditEvent::ShardSessionFinished {
                outcome: ShardSessionOutcome::Recovered,
                retry_count: 1,
                fallback_count: 1,
                ..
            }
        )));

    std::fs::remove_file(whole_artifact).expect("whole generated GGUF should be removable");
    std::fs::remove_file(first_artifact).expect("first generated GGUF should be removable");
    std::fs::remove_file(final_artifact).expect("final generated GGUF should be removable");
}
