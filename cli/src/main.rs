use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    synapseflow_cli::run().await
}
