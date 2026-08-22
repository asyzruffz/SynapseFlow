use crate::{DomainError, DomainResult};

/// Immutable, versioned reference to a signed model manifest.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModelReference(String);

impl ModelReference {
    pub fn parse(value: String) -> DomainResult<Self> {
        let valid = value.split_once("@sha256:").is_some_and(|(path, hash)| {
            path.starts_with("registry://")
                && path.len() > "registry://".len()
                && hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        }) && !value.contains(char::is_whitespace);
        if !valid {
            return Err(DomainError::InvalidReference);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the lower-case SHA-256 identity embedded in this reference.
    pub fn manifest_sha256(&self) -> &str {
        self.0.rsplit_once("@sha256:").map_or("", |(_, hash)| hash)
    }
}
