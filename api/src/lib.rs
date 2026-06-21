//! SynapseFlow API Library
//!
//! REST/gRPC endpoints for user prompts, model administration operations.

mod admin;
pub mod local_api; // User-facing inference endpoints (/v1/predict) // Admin ops: shard upload, peer health metrics
