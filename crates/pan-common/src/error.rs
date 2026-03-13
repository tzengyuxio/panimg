use serde::Serialize;

/// Exit codes for structured error reporting (POSIX-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(into = "u8")]
pub enum ExitCode {
    Success = 0,
    General = 1,
    InputFile = 2,
    OutputIssue = 3,
    Unsupported = 4,
    BadArgs = 5,
}

impl From<ExitCode> for u8 {
    fn from(code: ExitCode) -> u8 {
        code as u8
    }
}

/// Trait for domain-specific errors with structured output support.
pub trait StructuredError: std::fmt::Display + Serialize {
    /// Return the exit code for this error.
    fn exit_code(&self) -> ExitCode;

    /// Return a human-readable suggestion for resolving this error.
    fn suggestion(&self) -> &str;
}
