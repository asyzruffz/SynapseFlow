use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::json;
use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::{AuditEvent, AuditSink};

use crate::{AuditSettings, NodeError};

const ACTIVE_FILE: &str = "audit.log";

/// Node-local, synchronous audit sink with bounded rotated-file retention.
///
/// The sink deliberately stores only fields already accepted by `AuditEvent`.
/// It never accepts a prompt, generated text, credential, model path, or raw
/// token as an input. Each append is flushed to the filesystem before success.
pub struct RotatingAuditSink {
    settings: AuditSettings,
    active_path: PathBuf,
    bytes_written: Mutex<u64>,
    active_since: Mutex<SystemTime>,
    healthy: AtomicBool,
}

impl RotatingAuditSink {
    pub fn open(settings: AuditSettings) -> Result<Self, NodeError> {
        fs::create_dir_all(&settings.directory).map_err(|_| NodeError::AuditStorageUnavailable)?;
        restrict_directory(&settings.directory).map_err(|_| NodeError::AuditStorageUnavailable)?;
        let active_path = settings.directory.join(ACTIVE_FILE);
        let bytes_written = fs::metadata(&active_path).map_or(0, |metadata| metadata.len());
        let active_since = fs::metadata(&active_path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or_else(|_| SystemTime::now());
        let sink = Self {
            settings,
            active_path,
            bytes_written: Mutex::new(bytes_written),
            active_since: Mutex::new(active_since),
            healthy: AtomicBool::new(true),
        };
        sink.ensure_active_file()
            .map_err(|_| NodeError::AuditStorageUnavailable)?;
        Ok(sink)
    }

    /// Reports whether the last storage operation completed successfully.
    /// Composition uses this as a readiness/admission input in later steps.
    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }

    fn ensure_active_file(&self) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.active_path)?;
        restrict_file(&self.active_path)?;
        file.sync_data()
    }

    fn record_inner(&self, event: AuditEvent) -> std::io::Result<()> {
        let mut record = audit_record(event)?;
        record.push(b'\n');
        let mut bytes_written = self
            .bytes_written
            .lock()
            .map_err(|_| std::io::Error::other("audit lock unavailable"))?;
        let mut active_since = self
            .active_since
            .lock()
            .map_err(|_| std::io::Error::other("audit lock unavailable"))?;
        if should_rotate(
            *bytes_written,
            record.len() as u64,
            *active_since,
            self.settings.max_file_bytes,
            Duration::from_secs(self.settings.max_file_age_seconds),
        ) {
            self.rotate()?;
            *bytes_written = 0;
            *active_since = SystemTime::now();
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.active_path)?;
        restrict_file(&self.active_path)?;
        file.write_all(&record)?;
        file.sync_data()?;
        *bytes_written += record.len() as u64;
        Ok(())
    }

    fn rotate(&self) -> std::io::Result<()> {
        let oldest = self.settings.max_retained_files;
        let oldest_path = rotated_path(&self.settings.directory, oldest);
        if oldest_path.exists() {
            fs::remove_file(oldest_path)?;
        }
        for index in (1..oldest).rev() {
            let source = rotated_path(&self.settings.directory, index);
            if source.exists() {
                fs::rename(source, rotated_path(&self.settings.directory, index + 1))?;
            }
        }
        if self.active_path.exists() {
            fs::rename(&self.active_path, rotated_path(&self.settings.directory, 1))?;
        }
        self.ensure_active_file()
    }
}

fn should_rotate(
    bytes_written: u64,
    incoming_bytes: u64,
    active_since: SystemTime,
    max_file_bytes: u64,
    max_file_age: Duration,
) -> bool {
    let oversized = bytes_written > 0 && bytes_written + incoming_bytes > max_file_bytes;
    let expired = bytes_written > 0 && active_since.elapsed().is_ok_and(|age| age >= max_file_age);
    oversized || expired
}

impl AuditSink for RotatingAuditSink {
    fn record(&self, event: AuditEvent) -> DomainResult<()> {
        if !self.is_healthy() {
            return Err(DomainError::AuditUnavailable);
        }
        self.record_inner(event).map_err(|_| {
            self.healthy.store(false, Ordering::Release);
            DomainError::AuditUnavailable
        })
    }
}

fn audit_record(event: AuditEvent) -> std::io::Result<Vec<u8>> {
    let timestamp_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| std::io::Error::other("clock unavailable"))?
        .as_secs();
    let value = match event {
        AuditEvent::NodeSession(event) => json!({
            "at": timestamp_seconds,
            "kind": "node_session",
            "principal": event.principal.pseudonym().as_str(),
            "authorization": format!("{:?}", event.authorization),
            "admission": format!("{:?}", event.admission),
            "session_id": event.session_id.as_str(),
            "trace_id": event.trace_id.as_ref().map(|value| value.as_str()),
            "model": event.model.as_str(),
            "token_count": event.token_count,
            "failure": event.failure.map(|value| value.to_string()),
            "cancellation": event.cancellation.map(|value| format!("{:?}", value)),
        }),
        AuditEvent::GenerationStarted { model } => {
            json!({"at": timestamp_seconds, "kind": "generation_started", "model": model.as_str()})
        }
        AuditEvent::GenerationCompleted { model, token_count } => {
            json!({"at": timestamp_seconds, "kind": "generation_completed", "model": model.as_str(), "token_count": token_count})
        }
        AuditEvent::GenerationFailed { model } => {
            json!({"at": timestamp_seconds, "kind": "generation_failed", "model": model.as_str()})
        }
        AuditEvent::ManifestVerified {
            model,
            publisher_key_id,
        } => {
            json!({"at": timestamp_seconds, "kind": "manifest_verified", "model": model.as_str(), "publisher_key_id": publisher_key_id})
        }
        AuditEvent::ArtifactsCached {
            model,
            artifact_count,
        } => {
            json!({"at": timestamp_seconds, "kind": "artifacts_cached", "model": model.as_str(), "artifact_count": artifact_count})
        }
        AuditEvent::ModelAcquisitionFailed { model } => {
            json!({"at": timestamp_seconds, "kind": "model_acquisition_failed", "model": model.as_str()})
        }
        AuditEvent::ShardSessionFinished {
            model,
            shard,
            worker,
            session_id,
            outcome,
            retry_count,
            fallback_count,
        } => json!({
            "at": timestamp_seconds,
            "kind": "shard_session_finished",
            "model": model.as_str(),
            "shard": shard.as_str(),
            "worker": worker.as_str(),
            "session_id": session_id.as_str(),
            "outcome": format!("{:?}", outcome),
            "retry_count": retry_count,
            "fallback_count": fallback_count,
        }),
    };
    serde_json::to_vec(&value).map_err(|_| std::io::Error::other("audit serialization failed"))
}

fn rotated_path(directory: &Path, index: usize) -> PathBuf {
    directory.join(format!("audit.{index}.log"))
}

#[cfg(unix)]
fn restrict_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn restrict_directory(_: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_file(_: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use synapseflow_domain::ModelReference;
    use synapseflow_ports::{AuditEvent, AuditSink};

    use super::{AuditSettings, RotatingAuditSink};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "synapseflow-node-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn model() -> ModelReference {
        ModelReference::parse(format!(
            "registry://fixtures/audit@sha256:{}",
            "a".repeat(64)
        ))
        .expect("fixture model should be valid")
    }

    #[test]
    fn flushes_safe_records_and_rotates_with_bounded_retention() {
        let directory = temporary_directory("audit");
        let sink = RotatingAuditSink::open(AuditSettings {
            directory: directory.clone(),
            max_file_bytes: 1,
            max_file_age_seconds: 86_400,
            max_retained_files: 1,
        })
        .expect("temporary audit directory should open");

        sink.record(AuditEvent::GenerationFailed { model: model() })
            .expect("first audit record should persist");
        sink.record(AuditEvent::GenerationFailed { model: model() })
            .expect("second audit record should rotate and persist");

        assert!(sink.is_healthy());
        assert!(directory.join("audit.1.log").exists());
        assert!(fs::read_to_string(directory.join("audit.log"))
            .expect("active log should be readable")
            .contains("generation_failed"));
        fs::remove_dir_all(directory).expect("test should remove only its temporary directory");
    }

    #[test]
    fn rejects_a_directory_nested_under_a_regular_file() {
        let file = temporary_directory("file");
        fs::write(&file, "not a directory").expect("fixture file should be writable");
        let result = RotatingAuditSink::open(AuditSettings {
            directory: file.join("child"),
            max_file_bytes: 1,
            max_file_age_seconds: 86_400,
            max_retained_files: 1,
        });

        assert!(result.is_err());
        fs::remove_file(file).expect("test should remove only its fixture file");
    }

    #[test]
    fn rotates_an_active_file_that_exceeds_its_age_limit() {
        let directory = temporary_directory("audit-age");
        let sink = RotatingAuditSink::open(AuditSettings {
            directory: directory.clone(),
            max_file_bytes: 1_024 * 1_024,
            max_file_age_seconds: 1,
            max_retained_files: 1,
        })
        .expect("temporary audit directory should open");
        sink.record(AuditEvent::GenerationFailed { model: model() })
            .expect("first audit record should persist");
        *sink.active_since.lock().expect("audit lock") = UNIX_EPOCH;

        sink.record(AuditEvent::GenerationFailed { model: model() })
            .expect("expired audit file should rotate");

        assert!(directory.join("audit.1.log").exists());
        fs::remove_dir_all(directory).expect("test should remove only its temporary directory");
    }
}
