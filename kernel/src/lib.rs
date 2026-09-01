//! Headless application core shared by all SynapseFlow clients.
//!
//! Shells submit [`Event`] values, execute returned [`Effect`] requests, and
//! render the resulting [`ViewModel`]. The kernel contains no runtime,
//! transport, or UI implementation.

mod app;
pub mod effects;
mod event;
mod state;
mod view;

#[cfg(test)]
mod tests;

pub use app::SynapseFlow;
pub use crux_core::Core;
pub use effects::{Effect, GenerationCompletion, GenerationExecution};
pub use event::Event;
pub use view::ViewModel;
