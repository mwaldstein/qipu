use crate::pricing::get_model_pricing;
use chrono::Utc;

pub fn generate_run_id() -> String {
    let now = Utc::now();
    format!("run-{}", now.format("%Y%m%d-%H%M%S-%f"))
}

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
