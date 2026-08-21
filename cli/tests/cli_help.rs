use std::process::Command;

#[test]
fn help_is_available_without_a_model_or_developer_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_synapseflow"))
        .arg("--help")
        .output()
        .expect("CLI help command should start");

    assert!(output.status.success(), "CLI help should exit successfully");

    let stdout = String::from_utf8(output.stdout).expect("CLI help should be UTF-8");
    assert!(stdout.contains("SynapseFlow CLI"));
    assert!(stdout.contains("--model"));
}
