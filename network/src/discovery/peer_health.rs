//! Peer health tracking and liveness monitoring for P2P overlay network.
//!
//! Tracks peer status: online/offline/error with ping timeouts, identifies faulty peers

use anyhow::Result;
