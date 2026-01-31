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

    #[error("invalid {context}: {value}")]
    InvalidValue { context: String, value: String },

    #[error("{context} already exists: {value}")]
    AlreadyExists { context: String, value: String },

    #[error("{context} not found: {value}")]
    NotFound { context: String, value: String },

    #[error("unsupported {context}: {value} (supported: {supported})")]
    Unsupported {
        context: String,
        value: String,
        supported: String,
    },

    #[error("failed to {operation}: {reason}")]
    FailedOperation { operation: String, reason: String },

    #[error("failed to {operation} {target}: {reason}")]
    FailedOperationWithTarget {
        operation: String,
        target: String,
        reason: String,
    },

    #[error("field not found on note: {field}")]
    FieldNotFound { field: String, note_id: String },

    #[error("{0}")]
    Other(String),

    #[error("Index interrupted. Run `qipu index` to resume.")]
    Interrupted,
}

impl QipuError {
    /// Create an error for a failed database operation
    pub fn db_operation(operation: &str, error: impl std::fmt::Display) -> Self {
        QipuError::FailedOperation {
            operation: operation.to_string(),
            reason: error.to_string(),
        }
    }

    /// Create an error for a failed transaction operation
    pub fn transaction(operation: &str, error: impl std::fmt::Display) -> Self {
        QipuError::FailedOperation {
            operation: format!("{} transaction", operation),
            reason: error.to_string(),
        }
    }

    /// Create an error for a failed field extraction from database row
    pub fn field_extraction(field: &str, error: impl std::fmt::Display) -> Self {
        QipuError::FailedOperation {
            operation: format!("get {}", field),
            reason: error.to_string(),
        }
    }

    /// Create an error for a failed note operation
    pub fn note_operation(note_id: &str, operation: &str, error: impl std::fmt::Display) -> Self {
        QipuError::FailedOperationWithTarget {
            operation: operation.to_string(),
            target: format!("note {}", note_id),
            reason: error.to_string(),
        }
    }

    /// Create an error for a failed IO operation with context
    pub fn io_operation(
        operation: &str,
        path: impl std::fmt::Display,
        error: impl std::fmt::Display,
    ) -> Self {
        QipuError::FailedOperationWithTarget {
            operation: operation.to_string(),
            target: path.to_string(),
            reason: error.to_string(),
        }
    }

    /// Create an error for an invalid value or configuration
    pub fn invalid_value(context: &str, value: impl std::fmt::Display) -> Self {
        QipuError::InvalidValue {
            context: context.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an error for an entity that already exists
    pub fn already_exists(context: &str, value: impl std::fmt::Display) -> Self {
        QipuError::AlreadyExists {
            context: context.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an error for an entity that was not found
    pub fn not_found(context: &str, value: impl std::fmt::Display) -> Self {
        QipuError::NotFound {
            context: context.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an error for an unsupported value
    pub fn unsupported(
        context: &str,
        value: impl std::fmt::Display,
        supported: impl std::fmt::Display,
    ) -> Self {
        QipuError::Unsupported {
            context: context.to_string(),
            value: value.to_string(),
            supported: supported.to_string(),
        }
    }

    /// Create an error for a field not found on a note
    pub fn field_not_found(field: &str, note_id: &str) -> Self {
        QipuError::FieldNotFound {
            field: field.to_string(),
            note_id: note_id.to_string(),
        }
    }

    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            // Usage errors
            QipuError::UnknownFormat(_)
            | QipuError::DuplicateFormat
            | QipuError::UsageError(_)
            | QipuError::InvalidValue { .. }
            | QipuError::Unsupported { .. } => ExitCode::Usage,

            // Data/store errors
            QipuError::StoreNotFound { .. }
            | QipuError::InvalidStore { .. }
            | QipuError::NoteNotFound { .. }
            | QipuError::InvalidFrontmatter { .. }
            | QipuError::NotFound { .. }
            | QipuError::AlreadyExists { .. } => ExitCode::Data,

            // Generic failures
            QipuError::Io(_)
            | QipuError::Yaml(_)
            | QipuError::Json(_)
            | QipuError::Toml(_)
            | QipuError::FailedOperation { .. }
            | QipuError::FailedOperationWithTarget { .. }
            | QipuError::FieldNotFound { .. }
            | QipuError::Other(_)
            | QipuError::Interrupted => ExitCode::Failure,
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
            QipuError::InvalidValue { .. } => "invalid_value",
            QipuError::AlreadyExists { .. } => "already_exists",
            QipuError::NotFound { .. } => "not_found",
            QipuError::Unsupported { .. } => "unsupported",
            QipuError::FailedOperation { .. } => "failed_operation",
            QipuError::FailedOperationWithTarget { .. } => "failed_operation_with_target",
            QipuError::FieldNotFound { .. } => "field_not_found",
            QipuError::Other(_) => "other",
            QipuError::Interrupted => "interrupted",
        }
    }
}

/// Result type alias for qipu operations
pub type Result<T> = std::result::Result<T, QipuError>;
