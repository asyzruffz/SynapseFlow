use std::{net::SocketAddr, process::ExitCode};

use clap::Parser;
use synapseflow_domain::ModelReference;
use synapseflow_node::{build_verified_local_node, router, VerifiedLocalRuntimeArgs};

#[derive(Parser)]
#[command(
    name = "synapseflow-node",
    about = "SynapseFlow verified local API node"
)]
struct NodeArgs {
    /// Immutable manifest reference served by this local node.
    #[arg(long)]
    model: String,
    /// Loopback address for the local API.
    #[arg(long, default_value = "127.0.0.1:7878")]
    bind: SocketAddr,
    #[command(flatten)]
    runtime: VerifiedLocalRuntimeArgs,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = NodeArgs::parse();
    if !args.bind.ip().is_loopback() {
        eprintln!("SYN-NODE-001: node bind address must be loopback-only");
        return ExitCode::from(2);
    }
    let result = ModelReference::parse(args.model)
        .and_then(|reference| args.runtime.into_config().map(|config| (reference, config)))
        .and_then(|(reference, config)| build_verified_local_node(&reference, config));
    let node = match result {
        Ok(node) => node,
        Err(error) => {
            eprintln!("{}: {error}", error.code());
            return ExitCode::from(2);
        }
    };
    let listener = match tokio::net::TcpListener::bind(args.bind).await {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("SYN-NODE-002: unable to bind local API listener");
            return ExitCode::from(2);
        }
    };
    if axum::serve(listener, router(node)).await.is_err() {
        eprintln!("SYN-NODE-003: local API server failed");
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
