//! Error types for the asusctl backend.

/// Errors that can occur when interacting with asusctl.
#[derive(Debug, Clone)]
pub enum AsusctlError {
    /// asusctl binary not found
    NotInstalled,
    /// asusd service not running
    ServiceNotRunning,
    /// Command execution failed
    CommandFailed(String),
    /// Failed to parse command output
    ParseError(String),
}

impl std::fmt::Display for AsusctlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "asusctl is not installed"),
            Self::ServiceNotRunning => write!(f, "asusd service is not running"),
            Self::CommandFailed(msg) => write!(f, "Command failed: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for AsusctlError {}

/// Result type alias for asusctl operations.
pub type Result<T> = std::result::Result<T, AsusctlError>;
