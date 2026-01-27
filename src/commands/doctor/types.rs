use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::store::Store;
use serde::Serialize;

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Warning - store is functional but suboptimal
    Warning,
    /// Error - store has data integrity issues
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A single diagnostic issue
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// Issue severity
    pub severity: Severity,
    /// Issue category (e.g., "duplicate-id", "broken-link", "invalid-frontmatter")
    pub category: String,
    /// Human-readable description
    pub message: String,
    /// Affected note ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<String>,
    /// Affected file path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Can this issue be auto-fixed?
    pub fixable: bool,
}

/// Result of running doctor checks
#[derive(Debug, Clone, Serialize)]
pub struct DoctorResult {
    /// Total notes scanned
    pub notes_scanned: usize,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// All issues found
    pub issues: Vec<Issue>,
    /// Issues that were fixed (when --fix is used)
    pub fixed_count: usize,
}

impl DoctorResult {
    pub fn new() -> Self {
        DoctorResult {
            notes_scanned: 0,
            error_count: 0,
            warning_count: 0,
            issues: Vec::new(),
            fixed_count: 0,
        }
    }

    pub fn add_issue(&mut self, issue: Issue) {
        match issue.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.issues.push(issue);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

impl Default for DoctorResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for running doctor checks
///
/// Provides all possible inputs that checks might need.
/// Each check implementation extracts what it requires.
#[allow(dead_code)]
pub struct CheckContext<'a> {
    /// Store reference (for database operations)
    pub store: Option<&'a Store>,
    /// Parsed notes (for content validation)
    pub notes: Option<&'a [Note]>,
    /// Built index (for duplicate detection)
    pub index: Option<&'a Index>,
    /// Threshold for near-duplicate detection
    pub threshold: Option<f64>,
}

impl<'a> Default for CheckContext<'a> {
    fn default() -> Self {
        Self {
            store: None,
            notes: None,
            index: None,
            threshold: None,
        }
    }
}

impl<'a> CheckContext<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store(mut self, store: &'a Store) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_notes(mut self, notes: &'a [Note]) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_index(mut self, index: &'a Index) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

/// Trait for implementing doctor checks
///
/// This trait provides a uniform interface for all check implementations,
/// enabling dynamic registration and execution of checks.
pub trait DoctorCheck {
    /// Unique name identifying this check (e.g., "duplicate-id", "broken-link")
    fn name(&self) -> &str;

    /// Human-readable description of what this check validates
    fn description(&self) -> &str;

    /// Run the check with the provided context
    ///
    /// Each check implementation should:
    /// 1. Extract the inputs it needs from `ctx`
    /// 2. Add any issues found to `result`
    fn run(&self, ctx: &CheckContext<'_>, result: &mut DoctorResult);
}
