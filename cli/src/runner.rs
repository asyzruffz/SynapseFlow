use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use synapseflow_kernel::GenerationCompletion;

use crate::commands::RunCommand;
use crate::runtime::build_verified_local_generation_service;
use crate::shell::CliShell;

/// Runs the CLI presentation over its verified-local runtime composition.
pub(super) fn run(command: RunCommand) -> ExitCode {
    let result = command
        .into_parts()
        .and_then(|(request, output_path, json, config)| {
            let generation = build_verified_local_generation_service(&request.model, config)?;
            CliShell::new(generation)
                .execute(request)
                .map(|generation| (generation, output_path, json))
        });

    match result {
        Ok((generation, output_path, json)) => {
            if let Err(error) = present(&generation, output_path, json) {
                eprintln!("{}: {}", error.code(), error.message());
                return ExitCode::from(2);
            }
            eprintln!("session: {}", generation.session_id);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: {error}", error.code());
            ExitCode::from(2)
        }
    }
}

fn present(
    generation: &GenerationCompletion,
    output_path: Option<PathBuf>,
    json: bool,
) -> Result<(), CliOutputError> {
    let rendered = render_output(generation, json)?;
    match output_path {
        Some(path) => write_new_file(&path, &rendered),
        None => {
            print!("{rendered}");
            Ok(())
        }
    }
}

fn render_output(generation: &GenerationCompletion, json: bool) -> Result<String, CliOutputError> {
    if !json {
        return Ok(generation.output.text.clone());
    }
    serde_json::to_string(&serde_json::json!({
        "session_id": generation.session_id.to_string(),
        "output": {
            "text": generation.output.text.clone(),
            "tokens": generation.output.tokens.iter().map(|token| serde_json::json!({
                "id": token.id,
                "text": token.text.clone(),
            })).collect::<Vec<_>>(),
        }
    }))
    .map(|document| format!("{document}\n"))
    .map_err(|_| CliOutputError::OutputUnavailable)
}

pub(super) fn write_new_file(path: &Path, text: &str) -> Result<(), CliOutputError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| CliOutputError::OutputUnavailable)?;
    file.write_all(text.as_bytes())
        .map_err(|_| CliOutputError::OutputUnavailable)
}

pub(super) enum CliOutputError {
    OutputUnavailable,
}

impl CliOutputError {
    fn code(&self) -> &'static str {
        "SYN-CLI-001"
    }

    fn message(&self) -> &'static str {
        "unable to create the explicit output destination"
    }
}
