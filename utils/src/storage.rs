//! Local disk cache with automatic eviction policies for temporary tensor data.
//!
//! Responsibilities:
//! - Store large activation tensors temporarily using sled (embedded LSM)
//! - Implement LRU/size-based eviction when storage quota exceeded
