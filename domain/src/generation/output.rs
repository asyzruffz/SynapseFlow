/// One generated model token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedToken {
    pub id: u32,
    pub text: String,
}

/// Completed generation data, independent of the caller's output transport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationOutput {
    pub tokens: Vec<GeneratedToken>,
    pub text: String,
}

impl GenerationOutput {
    pub fn from_tokens(tokens: Vec<GeneratedToken>) -> Self {
        let text = tokens.iter().map(|token| token.text.as_str()).collect();
        Self { tokens, text }
    }
}
