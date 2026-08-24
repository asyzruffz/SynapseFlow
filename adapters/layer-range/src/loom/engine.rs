use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::ExecutionCancellation;

use crate::runtime::{LoomExecutionOutput, LoomExecutionRequest, LoomExecutor, LoomModelLayout};

use super::archive::LoomArchive;
use super::model::LoomModel;

/// CPU execution engine for Loom's Llama-specific layer-range backend.
pub struct LoomEngine {
    models: Mutex<HashMap<ModelKey, Arc<Mutex<LoomModel>>>>,
}

impl LoomEngine {
    pub fn new() -> Self {
        Self {
            models: Mutex::new(HashMap::new()),
        }
    }

    fn model_for(
        &self,
        artifact: &Path,
        request: &LoomExecutionRequest,
    ) -> DomainResult<Arc<Mutex<LoomModel>>> {
        let key = ModelKey {
            artifact: artifact.to_path_buf(),
            range_start: request.declared_range.start(),
            range_end: request.declared_range.end_exclusive(),
        };
        let mut models = self.models.lock().map_err(|_| DomainError::CacheFailure)?;
        if let Some(model) = models.get(&key) {
            return Ok(Arc::clone(model));
        }
        let model = Arc::new(Mutex::new(LoomModel::load(
            artifact,
            request.declared_range,
        )?));
        models.insert(key, Arc::clone(&model));
        Ok(model)
    }
}

impl Default for LoomEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LoomExecutor for LoomEngine {
    fn inspect(&self, artifact: &Path) -> DomainResult<LoomModelLayout> {
        Ok(LoomArchive::open(artifact)?.model_layout())
    }

    fn execute(
        &self,
        artifact: &Path,
        request: &LoomExecutionRequest,
        cancellation: &dyn ExecutionCancellation,
    ) -> DomainResult<LoomExecutionOutput> {
        if cancellation.is_cancelled() {
            return Err(DomainError::SessionCancelled);
        }
        let model = self.model_for(artifact, request)?;
        let mut model = model.lock().map_err(|_| DomainError::CacheFailure)?;
        let output = model.execute(request, cancellation);
        if output.is_err() || cancellation.is_cancelled() {
            model.discard_session(&request.session_id);
        }
        if cancellation.is_cancelled() {
            return Err(DomainError::SessionCancelled);
        }
        output
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ModelKey {
    artifact: PathBuf,
    range_start: u32,
    range_end: u32,
}
