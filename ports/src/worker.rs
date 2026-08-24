use synapseflow_domain::execution::FrameTarget;
use synapseflow_domain::{DomainError, DomainResult, ExecutionStrategy, ModelReference, ShardId};

/// Stable identifier for a loopback or future remote worker.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new(value: String) -> DomainResult<Self> {
        let valid = !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
        if !valid {
            return Err(DomainError::WorkerUnavailable);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Current worker health supplied by a directory adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerHealth {
    Healthy,
    Unavailable,
}

/// Immutable model/shard availability advertised by a worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardAvailability {
    pub model: ModelReference,
    pub shard: ShardId,
}

/// Static worker capability and availability record suitable for deterministic planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerCapability {
    id: WorkerId,
    health: WorkerHealth,
    strategies: Vec<ExecutionStrategy>,
    shards: Vec<ShardAvailability>,
}

impl WorkerCapability {
    pub fn new(
        id: WorkerId,
        health: WorkerHealth,
        strategies: Vec<ExecutionStrategy>,
        shards: Vec<ShardAvailability>,
    ) -> DomainResult<Self> {
        if strategies.is_empty() || shards.is_empty() || has_duplicates(&strategies, &shards) {
            return Err(DomainError::WorkerUnavailable);
        }
        Ok(Self {
            id,
            health,
            strategies,
            shards,
        })
    }

    pub fn id(&self) -> &WorkerId {
        &self.id
    }

    pub const fn health(&self) -> WorkerHealth {
        self.health
    }

    pub fn supports(&self, strategy: &ExecutionStrategy) -> bool {
        self.strategies.contains(strategy)
    }

    pub fn has_shard(&self, target: &FrameTarget) -> bool {
        self.shards
            .iter()
            .any(|shard| shard.model == target.model && shard.shard == target.shard)
    }
}

fn has_duplicates(strategies: &[ExecutionStrategy], shards: &[ShardAvailability]) -> bool {
    strategies
        .iter()
        .enumerate()
        .any(|(index, strategy)| strategies[index + 1..].contains(strategy))
        || shards.iter().enumerate().any(|(index, shard)| {
            shards[index + 1..]
                .iter()
                .any(|other| other.model == shard.model && other.shard == shard.shard)
        })
}

#[cfg(test)]
mod tests {
    use super::{ShardAvailability, WorkerCapability, WorkerHealth, WorkerId};
    use synapseflow_domain::{DomainError, ExecutionStrategy, ModelReference, ShardId};

    fn shard() -> ShardAvailability {
        ShardAvailability {
            model: ModelReference::parse(format!(
                "registry://fixtures/tinyllama@sha256:{}",
                "a".repeat(64)
            ))
            .expect("fixture model is valid"),
            shard: ShardId::new("first".to_owned()).expect("fixture shard is valid"),
        }
    }

    #[test]
    fn capabilities_require_unique_non_empty_strategy_and_shard_advertisements() {
        let worker = WorkerId::new("loopback-a".to_owned()).expect("worker identifier is valid");
        assert!(matches!(
            WorkerCapability::new(
                worker,
                WorkerHealth::Healthy,
                vec![
                    ExecutionStrategy::layer_range(),
                    ExecutionStrategy::layer_range()
                ],
                vec![shard()],
            ),
            Err(DomainError::WorkerUnavailable)
        ));
    }
}
