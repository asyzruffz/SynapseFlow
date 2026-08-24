use std::sync::Arc;

use synapseflow_domain::execution::FrameTarget;
use synapseflow_domain::{
    DomainError, DomainResult, ExecutionStrategy, ModelManifest,
    LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{PeerDirectory, WorkerCapability, WorkerHealth, WorkerId};

/// Deterministically selected primary and replica workers for one declared shard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardAssignment {
    pub target: FrameTarget,
    pub primary: WorkerId,
    pub replicas: Vec<WorkerId>,
}

/// Immutable route derived from a schema-v2 loopback-sharding manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionRoute {
    pub strategy: ExecutionStrategy,
    pub assignments: Vec<ShardAssignment>,
}

/// Plans static loopback workers using only the directory port and verified manifest data.
pub struct ShardPlanner {
    directory: Arc<dyn PeerDirectory>,
}

impl ShardPlanner {
    pub fn new(directory: Arc<dyn PeerDirectory>) -> Self {
        Self { directory }
    }

    pub fn plan(&self, manifest: &ModelManifest) -> DomainResult<ExecutionRoute> {
        if manifest.schema_version != LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION {
            return Err(DomainError::ManifestUnsupported);
        }
        let plan = manifest
            .execution_plan
            .as_ref()
            .ok_or(DomainError::ShardPlanInvalid)?;
        if !plan.strategy.is_layer_range() {
            return Err(DomainError::ExecutionStrategyUnsupported);
        }
        if plan.shards.len() != 2 {
            return Err(DomainError::ShardPlanInvalid);
        }

        let mut assignments = Vec::with_capacity(plan.shards.len());
        for shard in &plan.shards {
            let target = FrameTarget {
                model: manifest.reference.clone(),
                shard: shard.id().clone(),
            };
            let replicas =
                self.select_replicas(&target, &plan.strategy, shard.minimum_replicas())?;
            assignments.push(ShardAssignment {
                target,
                primary: replicas[0].clone(),
                replicas,
            });
        }

        Ok(ExecutionRoute {
            strategy: plan.strategy.clone(),
            assignments,
        })
    }

    fn select_replicas(
        &self,
        target: &FrameTarget,
        strategy: &ExecutionStrategy,
        minimum_replicas: u8,
    ) -> DomainResult<Vec<WorkerId>> {
        let mut candidates: Vec<WorkerCapability> = self
            .directory
            .replicas(target, strategy)?
            .into_iter()
            .filter(|worker| {
                worker.health() == WorkerHealth::Healthy
                    && worker.supports(strategy)
                    && worker.has_shard(target)
            })
            .collect();
        candidates.sort_by(|left, right| left.id().cmp(right.id()));
        candidates.dedup_by(|left, right| left.id() == right.id());
        if candidates.len() < usize::from(minimum_replicas) {
            return Err(DomainError::WorkerUnavailable);
        }
        Ok(candidates
            .into_iter()
            .map(|worker| worker.id().clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::ShardPlanner;
    use crate::sharding::test_support::{directory, manifest};
    use synapseflow_domain::DomainError;

    #[test]
    fn derives_two_deterministic_assignments_from_static_loopback_capabilities() {
        let manifest = manifest(1);
        let route = ShardPlanner::new(directory(&manifest))
            .plan(&manifest)
            .expect("fixture route should plan");

        assert_eq!(route.assignments.len(), 2);
        assert_eq!(route.assignments[0].primary.as_str(), "loopback-a");
        assert_eq!(route.assignments[0].replicas[1].as_str(), "loopback-b");
        assert_eq!(route.assignments[1].primary.as_str(), "loopback-b");
    }

    #[test]
    fn rejects_a_plan_without_the_declared_replica_requirement() {
        let manifest = manifest(3);

        assert!(matches!(
            ShardPlanner::new(directory(&manifest)).plan(&manifest),
            Err(DomainError::WorkerUnavailable)
        ));
    }
}
