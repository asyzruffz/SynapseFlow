use crate::AdmissionSettings;

/// HTTP admission limits checked before model acquisition can start.
#[derive(Clone, Copy, Debug)]
pub(super) struct SessionRequestLimits {
    pub(super) max_prompt_bytes: usize,
    pub(super) max_output_tokens: u16,
    pub(super) max_deadline_ms: u64,
}

impl From<&AdmissionSettings> for SessionRequestLimits {
    fn from(settings: &AdmissionSettings) -> Self {
        Self {
            max_prompt_bytes: settings.max_prompt_bytes,
            max_output_tokens: settings.max_output_tokens,
            max_deadline_ms: settings.max_deadline_ms,
        }
    }
}
