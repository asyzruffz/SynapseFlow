//! SynapseFlow Coord Library
//!
//! Orchestrator module: session planning, scheduling, and in-flight inference management.

pub mod planner; // Execution plan builder with latency-aware logic
mod session_manager; // In-flight sessions tracking
