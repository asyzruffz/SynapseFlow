use crate::LlamaCppBackend;

#[test]
fn cpu_runtime_initializes() {
    LlamaCppBackend::new().expect("the llama.cpp CPU runtime should initialize");
}
