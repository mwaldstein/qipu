use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use serde_json::Value;
use std::path::Path;

pub struct OpenCodeAdapter;

fn parse_token_usage_from_json(output: &str) -> Option<super::TokenUsage> {
    let lines: Vec<&str> = output
        .lines()
        .filter(|line| line.starts_with('{'))
        .collect();
    let mut total_input = 0;
    let mut total_output = 0;

    for line in lines {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if json.get("type") == Some(&Value::String("step_finish".to_string())) {
                if let Some(tokens) = json.get("part").and_then(|p| p.get("tokens")) {
                    let input = tokens.get("input").and_then(|v| v.as_u64()).unwrap_or(0);
                    let output = tokens.get("output").and_then(|v| v.as_u64()).unwrap_or(0);
                    let reasoning = tokens
                        .get("reasoning")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_input += input + reasoning;
                    total_output += output;
                }
            }
        }
    }

    if total_input > 0 || total_output > 0 {
        Some(super::TokenUsage {
            input: total_input as usize,
            output: total_output as usize,
        })
    } else {
        None
    }
}

impl ToolAdapter for OpenCodeAdapter {
    fn name(&self) -> &str {
        "opencode"
    }

    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("opencode", &["--version"], Path::new("."), 10) {
            Ok((output, _)) => {
                let version = output.trim().to_string();
                Ok(super::ToolStatus {
                    available: true,
                    version: Some(version),
                    authenticated: true, // opencode doesn't require auth check
                    budget_remaining: None,
                })
            }
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "OpenCode tool not found: {}",
                e
            ))),
        }
    }

    fn execute_task(
        &self,
        context: &super::TaskContext,
        work_dir: &Path,
        transcript_dir: &Path,
    ) -> Result<super::ExecutionResult, super::AdapterError> {
        use std::fs;
        use std::time::Instant;

        let start = Instant::now();
        let runner = SessionRunner::new();

        // Combine system and task prompts
        let full_prompt = format!("{}\n\n{}", context.system_prompt, context.task_prompt);

        let args = vec!["run", "--format", "json", &full_prompt];
        let timeout_secs = context.timeout.as_secs();

        let (output, exit_code) = runner
            .run_command("opencode", &args, work_dir, timeout_secs)
            .map_err(|e| {
                super::AdapterError::ExecutionFailed(format!("opencode execution failed: {}", e))
            })?;

        // Parse token usage from JSON output
        let token_usage = parse_token_usage_from_json(&output);

        // Write transcript
        let transcript_path = transcript_dir.join("transcript.raw.txt");
        fs::write(&transcript_path, &output).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write transcript: {}", e))
        })?;

        let duration = start.elapsed();

        Ok(super::ExecutionResult {
            exit_code,
            duration,
            token_usage,
            cost_estimate: None,
        })
    }

    fn estimate_cost(&self, prompt_tokens: usize) -> Option<super::CostEstimate> {
        // Estimate based on typical opencode pricing
        let input_cost = (prompt_tokens as f64) / 1_000_000.0 * 3.0; // $3/M tokens
        let output_cost = (prompt_tokens as f64 * 0.5) / 1_000_000.0 * 15.0; // $15/M tokens
        Some(super::CostEstimate {
            estimated_usd: input_cost + output_cost,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        match runner.run_command("opencode", &["--version"], Path::new("."), 10) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("OpenCode tool not found: {}", e)),
        }
    }

    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<super::TokenUsage>)> {
        let runner = SessionRunner::new();

        // Use 'opencode run' with JSON format for token extraction
        let mut args = vec!["run", "--format", "json"];
        if let Some(model) = model {
            args.push("--model");
            args.push(model);
        }
        args.push(&scenario.task.prompt);

        let (output, exit_code) = runner.run_command("opencode", &args, cwd, timeout_secs)?;
        let token_usage = parse_token_usage_from_json(&output);

        Ok((output, exit_code, None, token_usage))
    }
}
