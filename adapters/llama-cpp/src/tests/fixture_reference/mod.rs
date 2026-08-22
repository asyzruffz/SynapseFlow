//! Explicit fixture-backed reference-output acceptance test.

mod config;
mod runner;
mod vector;

#[test]
#[ignore = "requires a separately provisioned signed GGUF fixture and acceptance vector"]
fn fixture_reference_output_matches_accepted_vector() -> Result<(), String> {
    runner::assert_reference_output()
}
