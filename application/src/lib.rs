//! Application use cases composed exclusively from domain contracts and ports.

mod generation_service;

#[cfg(test)]
mod generation_service_tests;

pub use generation_service::GenerationService;
