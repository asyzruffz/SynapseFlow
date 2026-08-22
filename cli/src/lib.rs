//! Command-line presentation for the verified local generation workflow.

mod commands;
mod runner;

#[cfg(test)]
mod tests;

use std::process::ExitCode;

use clap::Parser;
use commands::{Cli, Command};

/// Parses one CLI command and maps its sanitized result to a process exit code.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        return ExitCode::SUCCESS;
    };
    match command {
        Command::Run(command) => runner::run(command),
    }
}
