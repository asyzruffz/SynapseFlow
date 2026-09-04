use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, SyncSender, TrySendError},
        Arc,
    },
    thread,
};

use synapseflow_ports::{TelemetryEvent, TelemetrySink};

use crate::NodeError;

/// Bounded data accepted by the node telemetry adapter.
///
/// It intentionally excludes principal, session, trace, prompt, token, and
/// model identifiers so an exporter cannot accidentally create unbounded or
/// sensitive telemetry dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryRecord {
    IdentityVerified,
    AuthorizationEvaluated { authorized: bool },
    AdmissionEvaluated { admitted: bool },
    SessionTerminal { failed: bool },
}

/// Export boundary for an OpenTelemetry-compatible implementation supplied by
/// deployment composition. Export failure is observable but never affects audit.
pub trait TelemetryExporter: Send + Sync + 'static {
    fn export(&self, record: TelemetryRecord) -> Result<(), NodeError>;
}

/// Non-blocking bounded telemetry sink. A full or failed exporter drops only
/// telemetry records and increments a counter; it never blocks request work.
pub struct BoundedTelemetrySink {
    sender: SyncSender<TelemetryRecord>,
    dropped_records: Arc<AtomicUsize>,
    export_failures: Arc<AtomicUsize>,
}

impl BoundedTelemetrySink {
    pub fn new(queue_capacity: usize, exporter: Arc<dyn TelemetryExporter>) -> Self {
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let export_failures = Arc::new(AtomicUsize::new(0));
        let worker_failures = export_failures.clone();
        thread::spawn(move || {
            while let Ok(record) = receiver.recv() {
                if exporter.export(record).is_err() {
                    worker_failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        Self {
            sender,
            dropped_records: Arc::new(AtomicUsize::new(0)),
            export_failures,
        }
    }

    pub fn dropped_records(&self) -> usize {
        self.dropped_records.load(Ordering::Relaxed)
    }

    pub fn export_failures(&self) -> usize {
        self.export_failures.load(Ordering::Relaxed)
    }

    fn enqueue(&self, record: TelemetryRecord) {
        match self.sender.try_send(record) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.dropped_records.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl TelemetrySink for BoundedTelemetrySink {
    fn record(&self, event: TelemetryEvent<'_>) {
        let record = match event {
            TelemetryEvent::IdentityVerified => TelemetryRecord::IdentityVerified,
            TelemetryEvent::AuthorizationEvaluated { decision } => {
                TelemetryRecord::AuthorizationEvaluated {
                    authorized: matches!(
                        decision,
                        synapseflow_domain::AuthorizationDecision::Authorized
                    ),
                }
            }
            TelemetryEvent::AdmissionEvaluated { decision } => {
                TelemetryRecord::AdmissionEvaluated {
                    admitted: matches!(decision, synapseflow_domain::AdmissionDecision::Admitted),
                }
            }
            TelemetryEvent::SessionTerminal { failure, .. } => TelemetryRecord::SessionTerminal {
                failed: failure.is_some(),
            },
        };
        self.enqueue(record);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use synapseflow_ports::TelemetrySink;

    use crate::NodeError;

    use super::{BoundedTelemetrySink, TelemetryExporter, TelemetryRecord};

    #[derive(Default)]
    struct RecordingExporter(Mutex<Vec<TelemetryRecord>>);

    impl TelemetryExporter for RecordingExporter {
        fn export(&self, record: TelemetryRecord) -> Result<(), NodeError> {
            self.0
                .lock()
                .map_err(|_| NodeError::TelemetryExportUnavailable)?
                .push(record);
            Ok(())
        }
    }

    #[test]
    fn exports_only_bounded_safe_telemetry() {
        let exporter = Arc::new(RecordingExporter::default());
        let sink = BoundedTelemetrySink::new(1, exporter.clone());
        sink.record(synapseflow_ports::TelemetryEvent::IdentityVerified);
        for _ in 0..10 {
            if !exporter.0.lock().expect("exporter lock").is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            exporter.0.lock().expect("exporter lock").as_slice(),
            &[TelemetryRecord::IdentityVerified]
        );
        assert_eq!(sink.export_failures(), 0);
    }
}
