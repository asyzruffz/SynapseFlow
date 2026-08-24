//! Deterministic bounded loopback transport that exercises SynapseFlow frame bytes.

mod faults;
mod network;
mod transport;
mod worker;

pub use faults::LoopbackFault;
pub use network::LoopbackNetwork;
pub use transport::{LoopbackEvent, LoopbackTransport};
pub use worker::{LoopbackWorker, ReceivedWorkerFrame};

#[cfg(test)]
mod tests;
