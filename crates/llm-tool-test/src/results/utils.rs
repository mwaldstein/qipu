//! Utility functions for result handling.
//!
//! Provides helper functions for generating run IDs, getting qipu
//! version information, and estimating costs from token usage.

use crate::pricing::get_model_pricing;
use chrono::Utc;

/// Generate a unique run ID based on current timestamp.
///
/// Format: `run-YYYYMMDD-HHMMSS-microseconds`
///
/// # Returns
///
/// A unique run ID string
///
/// # Example
///
/// ```rust
/// use llm_tool_test::results::generate_run_id;
///
/// let run_id = generate_run_id();
/// assert!(run_id.starts_with("run-"));
/// ```
pub fn generate_run_id() -> String {
    let now = Utc::now();
    format!("run-{}", now.format("%Y%m%d-%H%M%S-%f"))
}

/// Get the current qipu git commit hash.
///
/// Runs `git rev-parse HEAD` in the parent directory to get
/// the short (8 character) commit hash.
///
/// # Returns
///
/// * `Ok(String)` - Short commit hash on success
/// * `Ok("unknown")` - If git command fails or is not available
///
/// # Example
///
/// ```rust
/// use llm_tool_test::results::get_qipu_version;
///
/// let version = get_qipu_version().unwrap();
/// // Returns "unknown" or an 8-character hash
/// ```
pub fn get_qipu_version() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir("..")
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(commit[..8].to_string());
        }
    }

    Ok("unknown".to_string())
}
