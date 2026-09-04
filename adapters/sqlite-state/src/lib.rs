//! SQLite-backed single-node control-plane state.
//!
//! This adapter retains only presentation-safe session metadata and checkpoint
//! references. It never stores prompts, bearer credentials, or model payloads.

mod active_sessions;
mod database;
mod session_state;

pub use session_state::{SqliteNodeState, SqliteNodeStateSettings};
