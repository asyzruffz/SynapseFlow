use std::sync::Arc;

use synapseflow_application::GenerationService;
use synapseflow_domain::{DomainResult, GenerationOutput, GenerationRequest};
use uuid::Uuid;

/// A local node delegates every generation request to the shared application
/// service. Future HTTP and gRPC handlers must use this same boundary.
#[derive(Clone)]
pub struct LocalNode {
    generation: Arc<GenerationService>,
}

/// One completed local workflow invocation with an opaque session identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalGeneration {
    pub session_id: Uuid,
    pub output: GenerationOutput,
}

impl LocalNode {
    /// Creates a node with its fully composed application service.
    pub fn new(generation: GenerationService) -> Self {
        Self {
            generation: Arc::new(generation),
        }
    }

    /// Handles one validated local generation request.
    pub fn generate(&self, request: GenerationRequest) -> DomainResult<GenerationOutput> {
        self.generation.generate(request)
    }

    /// Executes the common local workflow and assigns its opaque session ID.
    pub fn execute(&self, request: GenerationRequest) -> DomainResult<LocalGeneration> {
        self.generate(request).map(|output| LocalGeneration {
            session_id: Uuid::new_v4(),
            output,
        })
    }
}
