pub mod amp;
pub mod claude_code;
pub mod mock;
pub mod opencode;

use crate::scenario::Scenario;
use std::path::Path;
use std::time::Duration;

/// Error type for adapter operations.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AdapterError {
    #[error("Tool not available: {0}")]
    NotAvailable(String),

    #[error("Tool not authenticated: {0}")]
    NotAuthenticated(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Budget exhausted")]
    BudgetExhausted,

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Status of a tool's availability.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolStatus {
    pub available: bool,
    pub version: Option<String>,
    pub authenticated: bool,
    pub budget_remaining: Option<f64>,
}

/// Context for executing a task.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskContext {
    pub system_prompt: String,
    pub task_prompt: String,
    pub timeout: Duration,
}

/// Token usage statistics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TokenUsage {
    pub input: usize,
    pub output: usize,
}

/// Cost estimate for a task.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CostEstimate {
    pub estimated_usd: f64,
}

/// Result of executing a task.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub duration: Duration,
    pub token_usage: Option<TokenUsage>,
    pub cost_estimate: Option<f64>,
}

/// Trait for tool adapters that execute LLM CLI tools.
#[allow(dead_code)]
pub trait ToolAdapter: Send + Sync {
    /// Tool identifier.
    fn name(&self) -> &str;

    /// Check if tool is installed and authenticated.
    fn is_available(&self) -> Result<ToolStatus, AdapterError>;

    /// Execute a task and capture transcript.
    fn execute_task(
        &self,
        context: &TaskContext,
        work_dir: &Path,
        transcript_dir: &Path,
    ) -> Result<ExecutionResult, AdapterError>;

    /// Estimate cost for a prompt (if possible).
    fn estimate_cost(&self, prompt_tokens: usize) -> Option<CostEstimate>;

    // Legacy methods for backward compatibility during migration
    /// Check if the tool is available and ready to use.
    fn check_availability(&self) -> anyhow::Result<()> {
        match self.is_available() {
            Ok(status) if status.available => Ok(()),
            Ok(status) if !status.authenticated => Err(anyhow::anyhow!("Tool not authenticated")),
            Ok(_) => Err(anyhow::anyhow!("Tool not available")),
            Err(e) => Err(e.into()),
        }
    }

    /// Run the tool with the given scenario in the specified working directory.
    /// Returns the tool output, exit code, and estimated cost in USD (if available).
    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>)>;
}
