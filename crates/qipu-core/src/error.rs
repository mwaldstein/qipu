//! Error types and exit codes for qipu
//!
//! Exit codes per spec (specs/cli-tool.md):
//! - 0: Success
//! - 1: Generic failure
//! - 2: Usage error (bad flags/args)
//! - 3: Data/store error (missing store, invalid frontmatter, etc.)

mod macros;

use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Context information for an error occurrence.
/// Captures the operational context when an error occurred,
/// including tracing span info, file location, and operation details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    /// The operation being performed when the error occurred
    pub operation: String,
    /// File location (if available) where the error originated
    pub file: Option<String>,
    /// Line number (if available) where the error originated
    pub line: Option<u32>,
    /// Current tracing span context at error time
    pub span: Option<String>,
    /// Additional contextual key-value data
    pub metadata: Vec<(String, String)>,
}

impl ErrorContext {
    /// Create a new error context for an operation
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            file: None,
            line: None,
            span: None,
            metadata: Vec::new(),
        }
    }

    /// Add file location information
    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// Add span context
    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// A single entry in an error chain representing one error
/// with its context and potential cause.
#[derive(Debug, Clone)]
pub struct ErrorChainEntry {
    /// The error message
    pub message: String,
    /// Error type identifier
    pub error_type: String,
    /// Context when the error occurred
    pub context: Option<ErrorContext>,
    /// Timestamp when error occurred (nanoseconds since epoch)
    pub timestamp_ns: u64,
}

impl ErrorChainEntry {
    /// Create a new error chain entry
    pub fn new(
        message: impl Into<String>,
        error_type: impl Into<String>,
        context: Option<ErrorContext>,
    ) -> Self {
        Self {
            message: message.into(),
            error_type: error_type.into(),
            context,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
        }
    }
}

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

    /// Chained error with context and cause chain
    #[error("{message}")]
    Chained {
        /// Error message
        message: String,
        /// Error type identifier
        error_type: String,
        /// Chain of errors leading to this one (newest first)
        chain: Arc<Vec<ErrorChainEntry>>,
        /// Operational context when error occurred
        context: Option<ErrorContext>,
    },
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

            // Chained errors - use the error type from the chain
            QipuError::Chained { error_type, .. } => {
                // Determine exit code based on error type string
                match error_type.as_str() {
                    "unknown_format" | "duplicate_format" | "usage_error" | "invalid_value"
                    | "unsupported" => ExitCode::Usage,
                    "store_not_found"
                    | "invalid_store"
                    | "note_not_found"
                    | "invalid_frontmatter"
                    | "not_found"
                    | "already_exists" => ExitCode::Data,
                    _ => ExitCode::Failure,
                }
            }
        }
    }

    /// Get the error type identifier
    fn error_type(&self) -> String {
        match self {
            QipuError::UnknownFormat(_) => "unknown_format".to_string(),
            QipuError::DuplicateFormat => "duplicate_format".to_string(),
            QipuError::UsageError(_) => "usage_error".to_string(),
            QipuError::StoreNotFound { .. } => "store_not_found".to_string(),
            QipuError::InvalidStore { .. } => "invalid_store".to_string(),
            QipuError::NoteNotFound { .. } => "note_not_found".to_string(),
            QipuError::InvalidFrontmatter { .. } => "invalid_frontmatter".to_string(),
            QipuError::Io(_) => "io_error".to_string(),
            QipuError::Yaml(_) => "yaml_error".to_string(),
            QipuError::Json(_) => "json_error".to_string(),
            QipuError::Toml(_) => "toml_error".to_string(),
            QipuError::InvalidValue { .. } => "invalid_value".to_string(),
            QipuError::AlreadyExists { .. } => "already_exists".to_string(),
            QipuError::NotFound { .. } => "not_found".to_string(),
            QipuError::Unsupported { .. } => "unsupported".to_string(),
            QipuError::FailedOperation { .. } => "failed_operation".to_string(),
            QipuError::FailedOperationWithTarget { .. } => {
                "failed_operation_with_target".to_string()
            }
            QipuError::FieldNotFound { .. } => "field_not_found".to_string(),
            QipuError::Other(_) => "other".to_string(),
            QipuError::Interrupted => "interrupted".to_string(),
            QipuError::Chained { error_type, .. } => error_type.clone(),
        }
    }

    /// Chain this error with additional context, creating a new chained error.
    /// The current error is added to the chain, and a new error message
    /// is provided as the primary cause.
    pub fn chain(self, message: impl Into<String>, context: Option<ErrorContext>) -> Self {
        let message = message.into();
        let error_type = self.error_type().to_string();

        // Build the chain from existing chain or create new one
        let mut chain = match &self {
            QipuError::Chained { chain, .. } => (**chain).clone(),
            _ => Vec::new(),
        };

        // Add the current error to the chain
        chain.push(ErrorChainEntry::new(
            self.to_string(),
            error_type.clone(),
            None, // Original error context is captured at chain creation time
        ));

        QipuError::Chained {
            message,
            error_type,
            chain: Arc::new(chain),
            context,
        }
    }

    /// Get the error chain if this is a chained error
    pub fn error_chain(&self) -> Option<&[ErrorChainEntry]> {
        match self {
            QipuError::Chained { chain, .. } => Some(chain.as_ref()),
            _ => None,
        }
    }

    /// Get the error context if available
    pub fn error_context(&self) -> Option<&ErrorContext> {
        match self {
            QipuError::Chained { context, .. } => context.as_ref(),
            _ => None,
        }
    }

    /// Convert error to JSON representation for structured error output.
    /// Includes chain and context information for chained errors.
    pub fn to_json(&self) -> serde_json::Value {
        let mut error_obj = serde_json::json!({
            "code": self.exit_code() as i32,
            "type": self.error_type(),
            "message": self.to_string(),
        });

        // Add chain information if this is a chained error
        if let QipuError::Chained { chain, context, .. } = self {
            // Build chain array
            let chain_array: Vec<serde_json::Value> = chain
                .iter()
                .map(|entry| {
                    let mut entry_obj = serde_json::json!({
                        "type": entry.error_type.clone(),
                        "message": entry.message.clone(),
                        "timestamp_ns": entry.timestamp_ns,
                    });
                    if let Some(ctx) = &entry.context {
                        entry_obj["context"] = serde_json::json!({
                            "operation": ctx.operation.clone(),
                            "file": ctx.file.clone(),
                            "line": ctx.line,
                            "span": ctx.span.clone(),
                            "metadata": ctx.metadata.iter().map(|(k,v)| serde_json::json!({"key": k, "value": v})).collect::<Vec<_>>(),
                        });
                    }
                    entry_obj
                })
                .collect();

            error_obj["chain"] = serde_json::json!(chain_array);

            // Add context information
            if let Some(ctx) = context {
                error_obj["context"] = serde_json::json!({
                    "operation": ctx.operation.clone(),
                    "file": ctx.file.clone(),
                    "line": ctx.line,
                    "span": ctx.span.clone(),
                    "metadata": ctx.metadata.iter().map(|(k,v)| serde_json::json!({"key": k, "value": v})).collect::<Vec<_>>(),
                });
            }
        }

        serde_json::json!({ "error": error_obj })
    }
}

/// Result type alias for qipu operations
pub type Result<T> = std::result::Result<T, QipuError>;
