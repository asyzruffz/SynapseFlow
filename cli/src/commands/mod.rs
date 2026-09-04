mod args;
mod run;
mod serve;

use clap::{Parser, Subcommand};

pub use run::RunCommand;
pub use serve::ServeCommand;

#[derive(Parser)]
#[command(
    name = "synapseflow",
    about = "SynapseFlow CLI — local inference client"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Submit a generation request through the application service.
    Run(Box<RunCommand>),
    /// Start the authenticated SynapseFlow node server.
    Serve(Box<ServeCommand>),
}
