use synapseflow_domain::{ExecutionStrategy, ShardSpec};

/// Validated strategy-specific requirements for one generic shard execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShardExecutionRequirements {
    LayerRange { shard: ShardSpec },
}

impl ShardExecutionRequirements {
    pub fn strategy(&self) -> ExecutionStrategy {
        match self {
            Self::LayerRange { .. } => ExecutionStrategy::layer_range(),
        }
    }

    pub fn shard(&self) -> &ShardSpec {
        match self {
            Self::LayerRange { shard } => shard,
        }
    }
}
