//! Local node boundary.
//!
//! HTTP and streaming adapters are added in milestone 4. This crate owns the
//! application-facing node surface now, without coupling generation logic to a
//! web framework.

mod local_node;

pub use local_node::LocalNode;
