//! Payload-free client-node control-plane values.

mod admission;
mod identity;
mod session;

pub use admission::{AdmissionDecision, AdmissionRejection};
pub use identity::{AuthenticatedPrincipal, GrantedScope, GrantedScopes, PrincipalPseudonym};
pub use session::{CancellationResult, PublicSessionId, PublicSessionState};
