use crate::commands::options::OutputFormat;

pub fn process_output(format: &OutputFormat) -> impl FnMut(&str) {
    match format {
        OutputFormat::Stdout => |token: &str| {
            print!("{}", token);
            use std::io::Write;
            if let Err(error) = std::io::stdout().flush() {
                eprintln!("failed to flush standard output: {error}");
            }
        },
        OutputFormat::File => |token: &str| {
            let out_path = match std::env::current_dir() {
                Ok(path) => path.join("synapseflow_output.txt"),
                Err(error) => {
                    eprintln!("failed to determine the output directory: {error}");
                    return;
                }
            };

            if let Err(error) = std::fs::write(&out_path, token) {
                eprintln!("failed to write {}: {error}", out_path.display());
                return;
            }

            println!("Wrote output to {}", out_path.display());
        },
    }
}
