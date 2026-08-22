use std::{fs::OpenOptions, io::Write, path::Path};

/// Writes a manifest once without replacing a previous acceptance artifact.
pub(crate) fn write_new(path: &Path, document: &[u8]) -> Result<(), String> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("cannot create manifest {}: {error}", path.display()))?;
    output
        .write_all(document)
        .map_err(|error| format!("cannot write manifest {}: {error}", path.display()))?;
    output
        .sync_all()
        .map_err(|error| format!("cannot flush manifest {}: {error}", path.display()))
}
