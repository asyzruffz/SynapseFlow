use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use synapseflow_domain::{CancellationResult, DomainError, DomainResult, PublicSessionId};
use synapseflow_ports::{ActiveSessionControl, ExecutionCancellation};

#[derive(Default)]
pub(super) struct ActiveSessions {
    cancellations: Mutex<BTreeMap<PublicSessionId, Arc<AtomicBool>>>,
}

impl ActiveSessionControl for ActiveSessions {
    fn activate(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<Arc<dyn ExecutionCancellation>> {
        let cancellation = Arc::new(AtomicBool::new(false));
        self.cancellations
            .lock()
            .map_err(|_| DomainError::SessionUnavailable)?
            .insert(session_id.clone(), cancellation.clone());
        Ok(Arc::new(Cancellation(cancellation)))
    }

    fn request_cancellation(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult> {
        if let Some(cancellation) = self
            .cancellations
            .lock()
            .map_err(|_| DomainError::SessionUnavailable)?
            .get(session_id)
        {
            cancellation.store(true, Ordering::Release);
        }
        Ok(CancellationResult::Requested)
    }

    fn deactivate(&self, session_id: &PublicSessionId) -> DomainResult<()> {
        self.cancellations
            .lock()
            .map_err(|_| DomainError::SessionUnavailable)?
            .remove(session_id);
        Ok(())
    }
}

struct Cancellation(Arc<AtomicBool>);

impl ExecutionCancellation for Cancellation {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
