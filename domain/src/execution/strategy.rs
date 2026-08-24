use crate::{DomainError, DomainResult};

/// Versioned execution-strategy identifier selected by a verified plan.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionStrategy(String);

impl ExecutionStrategy {
    /// The first supported strategy: ordered contiguous model-layer ranges.
    pub const LAYER_RANGE: &'static str = "layer_range_v1";

    pub fn new(value: String) -> DomainResult<Self> {
        let valid = !value.is_empty()
            && value.len() <= 64
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-' | b'.')
            });
        if !valid {
            return Err(DomainError::ExecutionStrategyUnsupported);
        }
        Ok(Self(value))
    }

    pub fn layer_range() -> Self {
        Self(Self::LAYER_RANGE.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_layer_range(&self) -> bool {
        self.0 == Self::LAYER_RANGE
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionStrategy;
    use crate::DomainError;

    #[test]
    fn accepts_a_versioned_strategy_identifier() {
        let strategy = ExecutionStrategy::new("block_chunk.v2".to_owned())
            .expect("versioned strategy should be valid");

        assert_eq!(strategy.as_str(), "block_chunk.v2");
        assert!(!strategy.is_layer_range());
    }

    #[test]
    fn rejects_an_unsafe_strategy_identifier() {
        assert!(matches!(
            ExecutionStrategy::new("layer range v1".to_owned()),
            Err(DomainError::ExecutionStrategyUnsupported)
        ));
    }
}
