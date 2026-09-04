use std::sync::Arc;

use crate::NodeDependencies;

/// Shared, fully composed application services for public HTTP handlers.
#[derive(Clone)]
pub(super) struct ApiState {
    pub(super) dependencies: Arc<NodeDependencies>,
}
