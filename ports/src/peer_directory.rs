use synapseflow_domain::{DomainResult, ModelReference};

/// Stores future worker capability and availability state.
pub trait PeerDirectory: Send + Sync {
    fn has_eligible_peer(&self, model: &ModelReference) -> DomainResult<bool>;
}
