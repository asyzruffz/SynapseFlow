//! In-flight inference sessions tracking and lifecycle management.
//!
//! Responsibilities:
//! - Track active prompts awaiting completion across all peers
//! - Handle timeouts, retries on failed frames/peers
//! - Manage checkpoints for rollback capability

use anyhow::Result;
