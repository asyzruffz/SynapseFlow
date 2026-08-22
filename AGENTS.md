# Repository working conventions

- Prefer small, responsibility-focused Rust modules over large `lib.rs` or binary entrypoint files.
- Keep `lib.rs` limited to module declarations, crate-level documentation, and intentional public re-exports.
- Separate domain types, validation, ports, application use cases, adapter implementations, CLI parsing, and composition roots into distinct files.
- Add or update focused tests beside the module/use case they protect; do not mix production implementation and broad test fixtures in one file when a dedicated test module is clearer.
- Preserve the architecture direction in `docs/architecture.md`: applications depend on application/domain/ports; adapters implement ports; domain and ports remain infrastructure-independent.
