pub use pan_common::error::{ExitCode, StructuredError};
use serde::Serialize;
use std::path::PathBuf;

/// Structured error type for panimg.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum PanimgError {
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf, suggestion: String },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf, suggestion: String },

    #[error("unsupported format: {format}")]
    UnsupportedFormat { format: String, suggestion: String },

    #[error("unknown format for: {path}")]
    UnknownFormat { path: PathBuf, suggestion: String },

    #[error("decode error: {message}")]
    DecodeError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        suggestion: String,
    },

    #[error("encode error: {message}")]
    EncodeError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        suggestion: String,
    },

    #[error("output already exists: {path}")]
    OutputExists { path: PathBuf, suggestion: String },

    #[error("invalid argument: {message}")]
    InvalidArgument { message: String, suggestion: String },

    #[error("resize error: {message}")]
    ResizeError { message: String, suggestion: String },

    #[error("io error: {message}")]
    IoError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        suggestion: String,
    },
}

impl StructuredError for PanimgError {
    fn exit_code(&self) -> ExitCode {
        match self {
            Self::FileNotFound { .. } | Self::PermissionDenied { .. } => ExitCode::InputFile,
            Self::OutputExists { .. } => ExitCode::OutputIssue,
            Self::UnsupportedFormat { .. } | Self::UnknownFormat { .. } => ExitCode::Unsupported,
            Self::InvalidArgument { .. } => ExitCode::BadArgs,
            Self::DecodeError { .. } => ExitCode::InputFile,
            Self::EncodeError { .. } | Self::IoError { .. } => ExitCode::OutputIssue,
            Self::ResizeError { .. } => ExitCode::General,
        }
    }

    fn suggestion(&self) -> &str {
        match self {
            Self::FileNotFound { suggestion, .. }
            | Self::PermissionDenied { suggestion, .. }
            | Self::UnsupportedFormat { suggestion, .. }
            | Self::UnknownFormat { suggestion, .. }
            | Self::DecodeError { suggestion, .. }
            | Self::EncodeError { suggestion, .. }
            | Self::OutputExists { suggestion, .. }
            | Self::InvalidArgument { suggestion, .. }
            | Self::ResizeError { suggestion, .. }
            | Self::IoError { suggestion, .. } => suggestion,
        }
    }
}

pub type Result<T> = std::result::Result<T, PanimgError>;
