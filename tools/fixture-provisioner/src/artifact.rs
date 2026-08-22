use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use sha2::{Digest, Sha256};

/// Immutable size and digest measured from a local fixture file.
pub(crate) struct ArtifactFingerprint {
    pub(crate) size_bytes: u64,
    pub(crate) content_sha256: String,
}

impl ArtifactFingerprint {
    pub(crate) fn from_file(path: &Path) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|error| format!("cannot open fixture {}: {error}", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut digest = Sha256::new();
        let mut size_bytes = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];

        loop {
            let read = reader
                .read(&mut buffer)
                .map_err(|error| format!("cannot read fixture {}: {error}", path.display()))?;
            if read == 0 {
                break;
            }
            let read_size = u64::try_from(read)
                .map_err(|_| "fixture chunk size cannot be represented".to_owned())?;
            size_bytes = size_bytes
                .checked_add(read_size)
                .ok_or_else(|| "fixture size exceeds supported range".to_owned())?;
            digest.update(&buffer[..read]);
        }

        Ok(Self {
            size_bytes,
            content_sha256: hex_lower(&digest.finalize()),
        })
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
