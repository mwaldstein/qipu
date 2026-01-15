use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Result of an LLM user validation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the test passed
    pub passed: bool,
    /// Detailed validation message
    pub message: String,
    /// Store state validation details
    pub store_validation: StoreValidation,
    /// Test execution duration in seconds
    pub duration_secs: f64,
    /// Path to captured transcript
    pub transcript_path: Option<PathBuf>,
}

/// Validation details for the resulting store state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreValidation {
    /// Number of notes created
    pub note_count: usize,
    /// Number of links created
    pub link_count: usize,
    /// Whether the store has meaningful structure
    pub has_structure: bool,
    /// Whether task knowledge was captured
    pub captured_task: bool,
    /// Detailed analysis
    pub details: Vec<String>,
}

impl StoreValidation {
    /// Create an empty validation result
    pub fn empty() -> Self {
        Self {
            note_count: 0,
            link_count: 0,
            has_structure: false,
            captured_task: false,
            details: vec!["Store validation could not be performed".to_string()],
        }
    }

    /// Check if the store state is considered valid
    pub fn is_valid(&self) -> bool {
        self.captured_task && (self.has_structure || self.link_count > 0)
    }
}

/// Abstract interface for LLM tool adapters
pub trait ToolAdapter {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Execute a test task with the LLM tool
    fn execute_task(
        &self,
        task_prompt: &str,
        work_dir: &Path,
        transcript_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Check if the tool is available on the system
    fn is_available(&self) -> bool;
}

/// Configuration for LLM user validation tests
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Which tool adapter to use
    pub tool: String,
    /// Base directory for transcript storage
    pub transcript_base: PathBuf,
    /// Whether to keep transcripts after test completion
    pub keep_transcripts: bool,
    /// Timeout for test execution in seconds
    pub timeout_secs: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            tool: "opencode".to_string(),
            transcript_base: PathBuf::from("tests/transcripts"),
            keep_transcripts: true,
            timeout_secs: 300, // 5 minutes
        }
    }
}
