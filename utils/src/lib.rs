//! SynapseFlow Utils Library
//!
//! Shared utilities: serialization helpers and local storage management with eviction policies.

pub mod serializer; // JSON/protobuf encoding helpers for frame protocol
mod storage; // Local disk cache using sled/zstd compression
