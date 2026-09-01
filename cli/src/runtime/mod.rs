//! Verified-local runtime configuration and composition owned by the CLI.

mod composition;
mod config;

pub(crate) use composition::build_verified_local_generation_service;
pub(crate) use config::VerifiedLocalRuntimeConfig;
