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

/// Estimate cost in USD from token usage.
///
/// Uses pricing information from the pricing module to calculate
/// estimated cost based on input and output tokens.
///
/// # Arguments
///
/// * `model` - Model identifier (e.g., "gpt-4o", "claude-3-5-sonnet-20241022")
/// * `input_tokens` - Number of input tokens
/// * `output_tokens` - Number of output tokens
///
/// # Returns
///
/// Estimated cost in USD, or 0.0 if pricing information is unavailable
///
/// # Example
///
/// ```rust
/// use llm_tool_test::results::estimate_cost_from_tokens;
///
/// let cost = estimate_cost_from_tokens("gpt-4o", 1000, 500);
/// // Returns approximate cost based on current pricing
/// ```
pub fn estimate_cost_from_tokens(model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
    let Some(pricing) = get_model_pricing(model) else {
        return 0.0;
    };

    let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k_tokens;
    let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k_tokens;

    input_cost + output_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_cost_claude_sonnet() {
        let cost = estimate_cost_from_tokens("claude-3-5-sonnet-20241022", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 3.0;
        let expected_output_cost = (500.0 / 1000.0) * 15.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_claude_haiku() {
        let cost = estimate_cost_from_tokens("claude-3-5-haiku-20241022", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 0.8;
        let expected_output_cost = (500.0 / 1000.0) * 4.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_gpt4o() {
        let cost = estimate_cost_from_tokens("gpt-4o", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 2.5;
        let expected_output_cost = (500.0 / 1000.0) * 10.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_unknown_model() {
        let cost = estimate_cost_from_tokens("unknown-model", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_cost_amp_smart() {
        let cost = estimate_cost_from_tokens("smart", 1000, 500);
        let expected_input_cost = (4000.0 / 4.0 / 1000.0) * 3.0;
        let expected_output_cost = (2000.0 / 4.0 / 1000.0) * 15.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_amp_free() {
        let cost = estimate_cost_from_tokens("free", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_cost_case_insensitive() {
        let cost1 = estimate_cost_from_tokens("GPT-4O", 1000, 500);
        let cost2 = estimate_cost_from_tokens("gpt-4o", 1000, 500);
        assert!((cost1 - cost2).abs() < 0.001);
    }
}
