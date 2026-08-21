//! Frame structure for activation streaming between peers with headers, payloads, checksums and control messages.

use serde::{Deserialize, Serialize};

/// A single frame transmitted over the transport layer (QUIC or IPC).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutboundFrame {
    /// Session identifier linking to active inference request
    pub session_id: String, // e.g., "sess-0x1a3f"

    /// Which frame within a batch/request sequence (starting at index 0)
    pub seq_index: u64, // For ordering multiple activations in one forward pass

    /// Total frames expected for this complete message
    pub total_frames: Option<u32>, // None = single-activation request

    /// DType of activation tensor (f16/i8/f32/etc.)
    #[serde(rename = "dtype")] // Use f16, i8, u16, etc.
    pub dtype_name: String,

    /// Shape dimensions in order [B,L,H] or whatever model uses
    pub shape_dims: Vec<u64>,

    /// SHA256 checksum for payload integrity check before decompressing
    //#[serde(serialize_with = "sha_to_hex")] // TODO: Serialize hex string form
    pub sha256_raw: Option<Vec<u8>>, // Actual bytes to avoid encoding cost when serializing JSON/protobuf

    /// Whether this frame is compressed and how (none, zstd level N)
    #[serde(default)]
    pub compression_level: CompressionLevel,

    /// Optional control message flag (ACK/NACK/RESEND/heartbeat/etc.)

    // TODO
    //#[serde(default = "default_false")] // false if no explicit ctrl msg;
    //pub ctrl_msg: ControlMessageOption,

    /// Binary tensor payload after any encoding/compression.
    data_bytes: Vec<u8>,
}

/// Compression strategy and its configured level for the activation data payload.
#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize)]
pub enum CompressionLevel {
    /// Send without compression (use sparingly or when needed)
    #[default]
    None,
    // TODO: Implement zstd at different levels later in Week 1-2
}
