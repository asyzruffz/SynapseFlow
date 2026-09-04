use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ErrorCode {
    OutputUnavailable,
    NodeConfigurationInvalid,
    NodeConfigurationUnavailable,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::OutputUnavailable => "SYN-CLI-001",
            Self::NodeConfigurationInvalid => "SYN-CLI-002",
            Self::NodeConfigurationUnavailable => "SYN-CLI-003",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub(super) enum CliError {
    #[error("unable to create the explicit output destination")]
    OutputUnavailable,
    #[error("node configuration is invalid")]
    NodeConfigurationInvalid,
    #[error("node configuration file is unavailable")]
    NodeConfigurationUnavailable,
}

impl CliError {
    pub(super) fn code(&self) -> ErrorCode {
        match self {
            Self::OutputUnavailable => ErrorCode::OutputUnavailable,
            Self::NodeConfigurationInvalid => ErrorCode::NodeConfigurationInvalid,
            Self::NodeConfigurationUnavailable => ErrorCode::NodeConfigurationUnavailable,
        }
    }
}
