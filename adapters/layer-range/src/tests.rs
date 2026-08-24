use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use synapseflow_domain::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
    RemainingDeadline, SessionId, StreamId, TensorDescriptor, TensorDtype,
};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DecodedFrame, DomainError, DomainResult, ExecutionStrategy,
    FrameCodec, FrameCompression, FrameExtension, LayerRange, ModelFormat, ModelManifest,
    ModelReference, ShardId, ShardPlan, ShardSpec, TokenizerDeclaration, TokenizerKind,
};
use synapseflow_ports::{
    ExecutionCancellation, NeverCancelled, ShardExecutionBackend, ShardExecutionOutput,
    ShardExecutionRequest, ShardExecutionRequirements, VerifiedModel,
};

use crate::{
    LayerRangeBackend, LayerRangeRuntime, NativeExecutionOutput, NativeLayerRangeRequest,
    NativeModelLayout,
};

struct RecordingRuntime {
    calls: Mutex<Vec<(String, NativeLayerRangeRequest)>>,
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

impl LayerRangeRuntime for RecordingRuntime {
    fn inspect(&self, _: &Path) -> DomainResult<NativeModelLayout> {
        Ok(NativeModelLayout {
            layer_count: 2,
            activation_width: 3,
            vocabulary_size: 5,
        })
    }

    fn execute(
        &self,
        artifact: &Path,
        request: &NativeLayerRangeRequest,
        _: &dyn ExecutionCancellation,
    ) -> DomainResult<NativeExecutionOutput> {
        self.calls
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .push((artifact.display().to_string(), request.clone()));
        if request.final_stage || self.invalid_first_output {
            Ok(NativeExecutionOutput::FinalLogits(vec![0.5; 5]))
        } else {
            Ok(NativeExecutionOutput::Boundary(vec![0.25, 0.5, 0.75]))
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
            runtime_profile: Some("llama-layer-range-v1".to_owned()),
        },
        vec![
            "C:/verified/first.gguf".into(),
            "C:/verified/second.gguf".into(),
        ],
    )
    .expect("verified fixture paths bind to declared artifacts")
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

fn frame(target: FrameTarget, dtype: TensorDtype, payload: Vec<u8>) -> DecodedFrame {
    let elements = u32::try_from(payload.len() / 4).expect("fixture length fits u32");
    let envelope = FrameEnvelope::new(
        FrameProtocolVersion::current(),
        FrameMessageType::Data,
        SessionId::new("session-00000001".to_owned()).expect("fixture session is valid"),
        StreamId::new(1).expect("fixture stream is valid"),
        FrameSequence::initial(),
        target,
        Some(TensorDescriptor::new(dtype, vec![elements]).expect("fixture tensor is valid")),
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
    let backend = LayerRangeBackend::with_runtime(runtime.clone());
    let first = request(
        &model,
        0,
        frame(
            target(&model, "first"),
            TensorDtype::U32,
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
fn rejects_wrong_stage_dtype_invalid_native_output_and_cancellation() {
    let model = model();
    let valid_input = frame(
        target(&model, "first"),
        TensorDtype::U32,
        7_u32.to_le_bytes().to_vec(),
    );
    let execution_request = request(&model, 0, valid_input.clone());
    let backend = LayerRangeBackend::with_runtime(Arc::new(RecordingRuntime {
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
            0.0_f32.to_le_bytes().to_vec(),
        ),
    );
    assert!(matches!(
        backend.execute(&model, &wrong_dtype, &NeverCancelled),
        Err(DomainError::FrameDtypeUnsupported)
    ));
}
