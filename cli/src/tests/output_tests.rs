use std::path::Path;

use crate::runner::write_new_file;

#[test]
fn explicit_output_refuses_to_overwrite_an_existing_file() {
    let result = write_new_file(Path::new("Cargo.toml"), "must not overwrite");

    assert!(result.is_err());
}
