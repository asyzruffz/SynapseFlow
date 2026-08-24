use crate::{DomainError, DomainResult};

use super::ExecutionStrategy;

/// Stable shard identifier within one immutable execution plan.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ShardId(String);

impl ShardId {
    pub fn new(value: String) -> DomainResult<Self> {
        let valid = !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
        if !valid {
            return Err(DomainError::ShardPlanInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Half-open contiguous range of transformer block indices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerRange {
    start: u32,
    end_exclusive: u32,
}

impl LayerRange {
    pub fn new(start: u32, end_exclusive: u32) -> DomainResult<Self> {
        if start >= end_exclusive {
            return Err(DomainError::ShardPlanInvalid);
        }
        Ok(Self {
            start,
            end_exclusive,
        })
    }

    pub const fn start(&self) -> u32 {
        self.start
    }

    pub const fn end_exclusive(&self) -> u32 {
        self.end_exclusive
    }

    pub const fn contains(&self, layer: u32) -> bool {
        layer >= self.start && layer < self.end_exclusive
    }
}

/// One strategy-specific shard assignment inside an immutable plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardSpec {
    pub id: ShardId,
    pub layer_range: LayerRange,
}

/// Ordered, complete layer-range plan for one model execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardPlan {
    pub strategy: ExecutionStrategy,
    pub shards: Vec<ShardSpec>,
}

impl ShardPlan {
    pub fn new(strategy: ExecutionStrategy, shards: Vec<ShardSpec>) -> DomainResult<Self> {
        if !strategy.is_layer_range_v1() || shards.is_empty() {
            return Err(DomainError::ShardPlanInvalid);
        }

        let mut expected_start = 0_u32;
        for shard in &shards {
            if shard.layer_range.start() != expected_start {
                return Err(DomainError::ShardPlanInvalid);
            }
            expected_start = shard.layer_range.end_exclusive();
        }

        Ok(Self { strategy, shards })
    }

    pub fn total_layers(&self) -> u32 {
        self.shards
            .last()
            .map_or(0, |shard| shard.layer_range.end_exclusive())
    }
}

#[cfg(test)]
mod tests {
    use super::{LayerRange, ShardId, ShardPlan, ShardSpec};
    use crate::{DomainError, ExecutionStrategy};

    fn shard(id: &str, start: u32, end_exclusive: u32) -> ShardSpec {
        ShardSpec {
            id: ShardId::new(id.to_owned()).expect("test shard identifier should be valid"),
            layer_range: LayerRange::new(start, end_exclusive)
                .expect("test layer range should be valid"),
        }
    }

    #[test]
    fn accepts_complete_ordered_layer_ranges() {
        let plan = ShardPlan::new(
            ExecutionStrategy::layer_range_v1(),
            vec![shard("first", 0, 11), shard("second", 11, 22)],
        )
        .expect("ordered plan should be valid");

        assert_eq!(plan.total_layers(), 22);
        assert!(plan.shards[1].layer_range.contains(21));
    }

    #[test]
    fn rejects_a_gapped_or_unsupported_plan() {
        assert!(matches!(
            ShardPlan::new(
                ExecutionStrategy::layer_range_v1(),
                vec![shard("first", 0, 11), shard("second", 12, 22)],
            ),
            Err(DomainError::ShardPlanInvalid)
        ));
        assert!(matches!(
            ShardPlan::new(
                ExecutionStrategy::new("tensor_parallel_v1".to_owned())
                    .expect("strategy identifier should be valid"),
                vec![shard("first", 0, 22)],
            ),
            Err(DomainError::ShardPlanInvalid)
        ));
    }
}
