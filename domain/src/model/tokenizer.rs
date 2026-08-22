/// Tokenizer declaration carried by an immutable model manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenizerDeclaration {
    pub kind: TokenizerKind,
    pub model: String,
}

impl TokenizerDeclaration {
    pub fn is_embedded_llama(&self) -> bool {
        self.kind == TokenizerKind::Embedded && self.model == "llama"
    }
}

/// Supported tokenizer storage modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenizerKind {
    Embedded,
}
