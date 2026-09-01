use crux_core::{App, Command};

use crate::{state::SynapseFlowState, view::ViewModel, Effect, Event};

/// The Crux application that coordinates one SynapseFlow client workflow.
#[derive(Default)]
pub struct SynapseFlow;

impl App for SynapseFlow {
    type Event = Event;
    type Model = SynapseFlowState;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut SynapseFlowState) -> Command<Effect, Event> {
        model.update(event)
    }

    fn view(&self, model: &SynapseFlowState) -> ViewModel {
        model.into()
    }
}
