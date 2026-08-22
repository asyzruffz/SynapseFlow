//! Local node boundary.
//!
//! This crate owns composition and local transports without coupling the
//! generation application service to a web framework or native backend.

#[cfg(feature = "http")]
mod http;
mod local_node;
mod local_runtime;

#[cfg(test)]
mod tests;

#[cfg(feature = "http")]
pub use http::router;
pub use local_node::{LocalGeneration, LocalNode};
pub use local_runtime::{build_verified_local_node, VerifiedLocalRuntimeConfig};

#[cfg(feature = "cli")]
pub use local_runtime::VerifiedLocalRuntimeArgs;
