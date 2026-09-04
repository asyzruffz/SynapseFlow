use std::sync::Arc;

use crate::{NodeDependencies, NodeWorkflowRegistry};

/// Shared, fully composed application services for public HTTP handlers.
#[derive(Clone)]
pub(super) struct ApiState {
    pub(super) dependencies: Arc<NodeDependencies>,
    pub(super) workflows: Arc<NodeWorkflowRegistry>,
}
