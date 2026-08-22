use synapseflow_application::GenerationService;
use synapseflow_domain::{DomainResult, GenerationOutput, GenerationRequest};

/// A local node delegates every generation request to the shared application
/// service. Future HTTP and gRPC handlers must use this same boundary.
pub struct LocalNode {
    generation: GenerationService,
}

impl LocalNode {
    /// Creates a node with its fully composed application service.
    pub fn new(generation: GenerationService) -> Self {
        Self { generation }
    }

    /// Handles one validated local generation request.
    pub fn generate(&self, request: GenerationRequest) -> DomainResult<GenerationOutput> {
        self.generation.generate(request)
    }
}
