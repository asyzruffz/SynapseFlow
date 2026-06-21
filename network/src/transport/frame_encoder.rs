//! Frame encoder for chunked activation transmission.
//!
//! Each frame structure:
//! - **Header**: session_id, frame_index, total_frames, tensor_dtype, shape, checksum
//! - **Payload**: compressed activation bytes (zstd)
//! - Control messages embedded as special frames

//use zstd; // Placeholder
