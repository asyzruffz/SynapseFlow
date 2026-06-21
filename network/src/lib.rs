//! SynapseFlow Network Library
//!
//! Provides QUIC transport, frame protocol encoding/decoding, and P2P peer discovery.

pub mod discovery; // libp2p DHT overlay for peer routing/health tracking
mod transport; // Transport layer (QUIC connections + framing)
