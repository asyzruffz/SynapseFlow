use synapseflow_domain::execution::{FrameMessageType, FrameTarget, RemainingDeadline};
use synapseflow_domain::{DecodedFrame, DomainError, DomainResult, ExecutionStrategy};

use crate::VerifiedModel;

use super::ShardExecutionRequirements;

/// A backend request containing only verified declaration metadata and a decoded boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardExecutionRequest {
    pub target: FrameTarget,
    pub next_target: Option<FrameTarget>,
    pub strategy: ExecutionStrategy,
    pub requirements: ShardExecutionRequirements,
    pub input: DecodedFrame,
    pub remaining_deadline: RemainingDeadline,
}

impl ShardExecutionRequest {
    /// Confirms the request is bound to an immutable verified manifest declaration.
    pub fn validate_for(&self, model: &VerifiedModel) -> DomainResult<()> {
        if self.strategy != self.requirements.strategy()
            || self.input.envelope.message_type != FrameMessageType::Data
            || self.input.envelope.target.model != self.target.model
            || model.manifest.reference != self.target.model
        {
            return Err(DomainError::ModelVersionMismatch);
        }

        let plan = model
            .manifest
            .execution_plan
            .as_ref()
            .ok_or(DomainError::ShardPlanInvalid)?;
        if plan.strategy != self.strategy {
            return Err(DomainError::ShardPlanInvalid);
        }
        let position = plan
            .shards
            .iter()
            .position(|shard| {
                shard == self.requirements.shard() && shard.id() == &self.target.shard
            })
            .ok_or(DomainError::ShardPlanInvalid)?;
        let expected_next = plan.shards.get(position + 1).map(|shard| FrameTarget {
            model: self.target.model.clone(),
            shard: shard.id().clone(),
        });
        if self.next_target != expected_next {
            return Err(DomainError::ShardPlanInvalid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ShardExecutionRequest;
    use crate::{ShardExecutionOutput, ShardExecutionRequirements, VerifiedModel};
    use synapseflow_domain::execution::{
        FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
        RemainingDeadline, SessionId, StreamId,
    };
    use synapseflow_domain::{
        ArtifactDescriptor, ArtifactId, DecodedFrame, DomainError, ExecutionStrategy, FrameCodec,
        FrameCompression, LayerRange, ModelFormat, ModelManifest, ModelReference, ShardId,
        ShardPlan, ShardSpec, TensorDescriptor, TensorDtype, TokenizerDeclaration, TokenizerKind,
        LOOM_RUNTIME_PROFILE,
    };

    fn fixture() -> (VerifiedModel, ShardExecutionRequest) {
        let reference = ModelReference::parse(format!(
            "registry://fixtures/tinyllama@sha256:{}",
            "a".repeat(64)
        ))
        .expect("fixture reference is valid");
        let shard = ShardSpec::new(
            ShardId::new("first".to_owned()).expect("fixture shard id is valid"),
            ArtifactId::new("weights".to_owned()).expect("fixture artifact id is valid"),
            LayerRange::new(0, 1).expect("fixture range is valid"),
            1,
        )
        .expect("fixture shard is valid");
        let manifest = ModelManifest {
            reference: reference.clone(),
            schema_version: 2,
            model_id: "fixture".to_owned(),
            model_version: "fixture-v1".to_owned(),
            format: ModelFormat::Gguf,
            architecture: "llama".to_owned(),
            quantization: "Q5_K_M".to_owned(),
            tokenizer: TokenizerDeclaration {
                kind: TokenizerKind::Embedded,
                model: "llama".to_owned(),
            },
            artifacts: vec![ArtifactDescriptor {
                id: ArtifactId::new("weights".to_owned()).expect("fixture artifact is valid"),
                uri: "https://fixtures.example/weights.gguf".to_owned(),
                content_sha256: format!("sha256:{}", "b".repeat(64)),
                size_bytes: 1,
            }],
            publisher_key_id: "ed25519:fixture".to_owned(),
            license: "MIT".to_owned(),
            provenance: "fixture".to_owned(),
            execution_plan: Some(
                ShardPlan::new(ExecutionStrategy::layer_range(), vec![shard.clone()])
                    .expect("fixture plan is valid"),
            ),
            runtime_profile: Some(LOOM_RUNTIME_PROFILE.to_owned()),
        };
        let target = FrameTarget {
            model: reference,
            shard: shard.id().clone(),
        };
        let envelope = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Data,
            SessionId::new("session-00000001".to_owned()).expect("fixture session is valid"),
            StreamId::new(1).expect("fixture stream is valid"),
            FrameSequence::initial(),
            target.clone(),
            Some(
                TensorDescriptor::new(TensorDtype::F32, vec![1]).expect("fixture tensor is valid"),
            ),
            RemainingDeadline::new(Duration::from_millis(100)).expect("fixture deadline is valid"),
        )
        .expect("fixture envelope is valid");
        let input: DecodedFrame = FrameCodec::decode(
            &FrameCodec::encode(&envelope, &[0, 0, 0, 0], FrameCompression::None, None)
                .expect("fixture frame encodes"),
        )
        .expect("fixture frame decodes");
        (
            VerifiedModel::without_cached_artifacts(manifest),
            ShardExecutionRequest {
                target,
                next_target: None,
                strategy: ExecutionStrategy::layer_range(),
                requirements: ShardExecutionRequirements::LayerRange { shard },
                input,
                remaining_deadline: RemainingDeadline::new(Duration::from_millis(100))
                    .expect("fixture deadline is valid"),
            },
        )
    }

    #[test]
    fn execution_contract_binds_input_and_output_to_the_verified_declaration() {
        let (model, request) = fixture();

        assert!(request.validate_for(&model).is_ok());
        assert!(ShardExecutionOutput::FinalLogits(request.input.clone())
            .validate_for(&request)
            .is_ok());

        let mut mismatched = request.clone();
        mismatched.target.shard = ShardId::new("other".to_owned()).expect("test shard is valid");
        assert!(matches!(
            mismatched.validate_for(&model),
            Err(DomainError::ShardPlanInvalid)
        ));
    }
}
