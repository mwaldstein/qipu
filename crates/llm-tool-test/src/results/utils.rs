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
