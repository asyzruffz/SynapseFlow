use std::process::Command;

#[test]
fn help_is_available_without_a_model_or_developer_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_synapseflow"))
        .args(["run", "--help"])
        .output()
        .expect("CLI help command should start");

    assert!(output.status.success(), "CLI help should exit successfully");

    let stdout = String::from_utf8(output.stdout).expect("CLI help should be UTF-8");
    assert!(stdout.contains("Submit a generation request"));
    assert!(stdout.contains("--model"));
    assert!(stdout.contains("--json"));
}

#[test]
fn invalid_reference_uses_a_stable_non_zero_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_synapseflow"))
        .args([
            "run",
            "--model",
            "not-a-reference",
            "--prompt",
            "test",
            "--manifest",
            "unused-manifest.json",
            "--artifact",
            "unused-artifact.gguf",
            "--cache-dir",
            "unused-cache",
            "--publisher-public-key",
            "unused-key",
        ])
        .output()
        .expect("CLI command should start");

    assert!(!output.status.success(), "invalid reference must fail");
    let stderr = String::from_utf8(output.stderr).expect("CLI error should be UTF-8");
    assert!(stderr.contains("SYN-MODEL-001"));
}
