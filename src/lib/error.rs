//! Error types and exit codes for qipu
//!
//! Exit codes per spec (specs/cli-tool.md):
//! - 0: Success
//! - 1: Generic failure
//! - 2: Usage error (bad flags/args)
//! - 3: Data/store error (invalid frontmatter, missing store, etc.)

use std::path::PathBuf;
use thiserror::Error;

/// Exit codes per qipu specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,
    /// Generic failure (1)
    Failure = 1,
    /// Usage error - bad flags/args (2)
    Usage = 2,
    /// Data/store error - missing store, invalid frontmatter (3)
    Data = 3,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

impl From<rusqlite::Error> for QipuError {
    fn from(err: rusqlite::Error) -> Self {
        QipuError::Other(err.to_string())
    }
}

/// Errors that can occur during qipu operations
#[derive(Error, Debug)]
pub enum QipuError {
    // Usage errors (exit code 2)
    #[error("unknown format: {0} (expected: human, json, or records)")]
    UnknownFormat(String),

    #[error("--format may only be specified once")]
    DuplicateFormat,

    #[error("{0}")]
    UsageError(String),

    // Data/store errors (exit code 3)
    #[error("store not found (searched from {search_root:?})")]
    StoreNotFound { search_root: PathBuf },

    #[error("invalid store: {reason}")]
    InvalidStore { reason: String },

    #[error("note not found: {id}")]
    NoteNotFound { id: String },

    #[error("invalid frontmatter in {path:?}: {reason}")]
    InvalidFrontmatter { path: PathBuf, reason: String },

    // Generic failures (exit code 1)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("{0}")]
    Other(String),
}

impl QipuError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            // Usage errors
            QipuError::UnknownFormat(_) | QipuError::DuplicateFormat | QipuError::UsageError(_) => {
                ExitCode::Usage
            }

            // Data/store errors
            QipuError::StoreNotFound { .. }
            | QipuError::InvalidStore { .. }
            | QipuError::NoteNotFound { .. }
            | QipuError::InvalidFrontmatter { .. } => ExitCode::Data,

            // Generic failures
            QipuError::Io(_)
            | QipuError::Yaml(_)
            | QipuError::Json(_)
            | QipuError::Toml(_)
            | QipuError::Other(_) => ExitCode::Failure,
        }
    }

    /// Convert error to JSON representation for structured error output
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.exit_code() as i32,
                "type": self.error_type(),
                "message": self.to_string(),
            }
        })
    }

    /// Get the error type identifier
    fn error_type(&self) -> &'static str {
        match self {
            QipuError::UnknownFormat(_) => "unknown_format",
            QipuError::DuplicateFormat => "duplicate_format",
            QipuError::UsageError(_) => "usage_error",
            QipuError::StoreNotFound { .. } => "store_not_found",
            QipuError::InvalidStore { .. } => "invalid_store",
            QipuError::NoteNotFound { .. } => "note_not_found",
            QipuError::InvalidFrontmatter { .. } => "invalid_frontmatter",
            QipuError::Io(_) => "io_error",
            QipuError::Yaml(_) => "yaml_error",
            QipuError::Json(_) => "json_error",
            QipuError::Toml(_) => "toml_error",
            QipuError::Other(_) => "other",
        }
    }
}

/// Result type alias for qipu operations
pub type Result<T> = std::result::Result<T, QipuError>;
