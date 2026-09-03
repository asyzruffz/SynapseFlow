use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use synapseflow_domain::{DomainError, DomainResult, GenerationEvent, PublicSessionId};
use synapseflow_kernel::{Core, Event, SynapseFlow, ViewModel};

/// Delivers a presentation-safe workflow view to one node client subscriber.
pub trait WorkflowSubscriber: Send + Sync {
    fn publish(&self, session_id: &PublicSessionId, view: &ViewModel) -> DomainResult<()>;
}

struct ActiveWorkflow {
    core: Arc<Mutex<Core<SynapseFlow>>>,
    subscribers: Vec<Arc<dyn WorkflowSubscriber>>,
}

/// Composition-root-owned bridge between active client workflows and node subscribers.
///
/// This registry holds no authority over durable session state, ownership,
/// authorization, cancellation, retries, checkpoints, or cleanup. Those remain
/// exclusively in the application session store and session manager.
#[derive(Default)]
pub struct NodeWorkflowRegistry {
    workflows: Mutex<BTreeMap<PublicSessionId, ActiveWorkflow>>,
}

impl NodeWorkflowRegistry {
    pub fn insert(&self, session_id: PublicSessionId, core: Core<SynapseFlow>) -> DomainResult<()> {
        let mut workflows = self
            .workflows
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)?;
        if workflows.contains_key(&session_id) {
            return Err(DomainError::DuplicateWork);
        }
        workflows.insert(
            session_id,
            ActiveWorkflow {
                core: Arc::new(Mutex::new(core)),
                subscribers: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn subscribe(
        &self,
        session_id: &PublicSessionId,
        subscriber: Arc<dyn WorkflowSubscriber>,
    ) -> DomainResult<()> {
        let mut workflows = self
            .workflows
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)?;
        let workflow = workflows
            .get_mut(session_id)
            .ok_or(DomainError::SessionUnavailable)?;
        workflow.subscribers.push(subscriber);
        Ok(())
    }

    /// Applies a session event to its client workflow and publishes its safe view.
    pub fn deliver(
        &self,
        session_id: &PublicSessionId,
        event: GenerationEvent,
    ) -> DomainResult<()> {
        let (core, subscribers) = {
            let workflows = self
                .workflows
                .lock()
                .map_err(|_| DomainError::PersistenceFailure)?;
            let workflow = workflows
                .get(session_id)
                .ok_or(DomainError::SessionUnavailable)?;
            (workflow.core.clone(), workflow.subscribers.clone())
        };
        let view = {
            let core = core.lock().map_err(|_| DomainError::PersistenceFailure)?;
            core.process_event(Event::GenerationEvent {
                session_id: session_id.clone(),
                event,
            });
            core.view()
        };
        for subscriber in subscribers {
            subscriber.publish(session_id, &view)?;
        }
        Ok(())
    }

    pub fn remove(&self, session_id: &PublicSessionId) -> DomainResult<()> {
        let mut workflows = self
            .workflows
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)?;
        workflows
            .remove(session_id)
            .map(|_| ())
            .ok_or(DomainError::SessionUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use synapseflow_domain::{DomainError, DomainResult, PublicSessionId};
    use synapseflow_kernel::{Core, SynapseFlow, ViewModel};

    use super::{NodeWorkflowRegistry, WorkflowSubscriber};

    #[derive(Default)]
    struct Subscriber(Mutex<Vec<ViewModel>>);

    impl WorkflowSubscriber for Subscriber {
        fn publish(&self, _: &PublicSessionId, view: &ViewModel) -> DomainResult<()> {
            self.0
                .lock()
                .map_err(|_| DomainError::PersistenceFailure)?
                .push(view.clone());
            Ok(())
        }
    }

    fn session_id() -> PublicSessionId {
        PublicSessionId::new("application-session-0001".to_owned())
            .expect("fixture session should be valid")
    }

    #[test]
    fn retains_one_kernel_and_bridges_safe_views_to_subscribers() {
        let registry = NodeWorkflowRegistry::default();
        let session_id = session_id();
        let subscriber = Arc::new(Subscriber::default());

        registry
            .insert(session_id.clone(), Core::<SynapseFlow>::new())
            .expect("workflow should register");
        registry
            .subscribe(&session_id, subscriber.clone())
            .expect("subscriber should attach");
        registry
            .deliver(&session_id, synapseflow_domain::GenerationEvent::Cancelled)
            .expect("workflow event should bridge");

        assert_eq!(subscriber.0.lock().expect("subscriber lock").len(), 1);
        registry
            .remove(&session_id)
            .expect("workflow should remove");
        assert!(matches!(
            registry.remove(&session_id),
            Err(DomainError::SessionUnavailable)
        ));
    }
}
