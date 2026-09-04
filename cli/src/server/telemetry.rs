use synapseflow_node::{NodeError, TelemetryExporter, TelemetryRecord};

pub(super) struct StderrTelemetryExporter;

impl TelemetryExporter for StderrTelemetryExporter {
    fn export(&self, record: TelemetryRecord) -> Result<(), NodeError> {
        eprintln!("synapseflow telemetry: {record:?}");
        Ok(())
    }
}
