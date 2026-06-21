//! JSON and protobuf serialization helpers for frame protocol messages.
//!
//! Responsibilities:
//! - Serialize/deserialize tensor shapes, metadata in compact binary format

use serde::Deserialize; // Forward declaration
