use crate::ErrorCode;

use super::GeneratedToken;

/// Ordered, framework-independent output from one generation session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenerationEvent {
    Token(GeneratedToken),
    Completed { token_count: usize },
    Cancelled,
    Failed { code: ErrorCode },
}

impl GenerationEvent {
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Cancelled | Self::Failed { .. }
        )
    }
}

/// Terminal result returned by a backend after it has delivered ordered tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationTerminal {
    Completed { token_count: usize },
    Cancelled,
}
