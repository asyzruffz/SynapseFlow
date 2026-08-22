//! SynapseFlow command-line application boundary.

mod commands;
mod composition;

use std::process::ExitCode;

use clap::Parser;
use commands::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        return ExitCode::SUCCESS;
    };

    match command {
        Command::Run(command) => run(command),
    }
}

fn run(command: commands::RunCommand) -> ExitCode {
    let result = command
        .into_request()
        .and_then(|request| composition::in_memory_generation_service().generate(request));

    match result {
        Ok(output) => {
            print!("{}", output.text);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: {error}", error.code());
            ExitCode::from(2)
        }
    }
}
