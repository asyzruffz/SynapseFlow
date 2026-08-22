//! Native runtime configuration and composition for one verified local model.

mod composition;
mod config;

pub use composition::build_verified_local_node;
pub use config::VerifiedLocalRuntimeConfig;

#[cfg(feature = "cli")]
pub use config::VerifiedLocalRuntimeArgs;
