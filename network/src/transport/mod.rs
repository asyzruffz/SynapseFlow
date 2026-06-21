//! Transport layer for peer-to-peer activation streaming over QUIC.
//!
//! Responsibilities:
//! - Connection management and pooling via `quinn` endpoint manager
//! - Frame encoding (header + payload) with compression support
//! - Decoding frames from network buffer
//! - Control messages (ACK/NACK/RETRY/CANCEL/HEARTBEAT)

mod frame;
mod frame_decoder;
mod frame_encoder;
pub mod transport_manager; // Exported sub-module for connection pooling
