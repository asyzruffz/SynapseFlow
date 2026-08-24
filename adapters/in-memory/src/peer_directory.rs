use synapseflow_domain::execution::FrameTarget;
use synapseflow_domain::{DomainResult, ExecutionStrategy};
use synapseflow_ports::{PeerDirectory, WorkerCapability, WorkerId};

/// Deterministic static worker directory for application-service tests.
pub struct InMemoryPeerDirectory {
    workers: Vec<WorkerCapability>,
}

impl InMemoryPeerDirectory {
    pub fn new(workers: Vec<WorkerCapability>) -> Self {
        Self { workers }
    }
}

impl PeerDirectory for InMemoryPeerDirectory {
    fn worker(&self, worker: &WorkerId) -> DomainResult<Option<WorkerCapability>> {
        Ok(self
            .workers
            .iter()
            .find(|candidate| candidate.id() == worker)
            .cloned())
    }

    fn replicas(
        &self,
        target: &FrameTarget,
        strategy: &ExecutionStrategy,
    ) -> DomainResult<Vec<WorkerCapability>> {
        Ok(self
            .workers
            .iter()
            .filter(|worker| worker.supports(strategy) && worker.has_shard(target))
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryPeerDirectory;
    use synapseflow_domain::execution::FrameTarget;
    use synapseflow_domain::{ExecutionStrategy, ModelReference, ShardId};
    use synapseflow_ports::{
        PeerDirectory, ShardAvailability, WorkerCapability, WorkerHealth, WorkerId,
    };

    #[test]
    fn returns_static_replicas_with_their_current_health() {
        let model = ModelReference::parse(format!(
            "registry://fixtures/tinyllama@sha256:{}",
            "a".repeat(64)
        ))
        .expect("fixture model is valid");
        let shard = ShardId::new("first".to_owned()).expect("fixture shard is valid");
        let worker = WorkerId::new("loopback-a".to_owned()).expect("worker identifier is valid");
        let capability = WorkerCapability::new(
            worker.clone(),
            WorkerHealth::Unavailable,
            vec![ExecutionStrategy::layer_range()],
            vec![ShardAvailability {
                model: model.clone(),
                shard: shard.clone(),
            }],
        )
        .expect("fixture capability is valid");
        let directory = InMemoryPeerDirectory::new(vec![capability]);

        let replicas = directory
            .replicas(
                &FrameTarget { model, shard },
                &ExecutionStrategy::layer_range(),
            )
            .expect("replica lookup should succeed");
        assert_eq!(replicas.len(), 1);
        assert_eq!(replicas[0].health(), WorkerHealth::Unavailable);
        assert_eq!(
            directory.worker(&worker).expect("lookup should succeed"),
            Some(replicas[0].clone())
        );
    }
}
