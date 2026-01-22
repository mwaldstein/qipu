use super::ToolAdapter;
use crate::results::estimate_cost;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::path::Path;

pub struct OpenCodeAdapter;

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

        let args = vec!["run", &full_prompt];
        let timeout_secs = context.timeout.as_secs();

        let (output, exit_code) = runner
            .run_command("opencode", &args, work_dir, timeout_secs)
            .map_err(|e| {
                super::AdapterError::ExecutionFailed(format!("opencode execution failed: {}", e))
            })?;

        // Write transcript
        let transcript_path = transcript_dir.join("transcript.raw.txt");
        fs::write(&transcript_path, &output).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write transcript: {}", e))
        })?;

        let duration = start.elapsed();
        let input_tokens = full_prompt.len() / 4; // rough estimate
        let output_tokens = output.len() / 4;

        Ok(super::ExecutionResult {
            exit_code,
            duration,
            token_usage: Some(super::TokenUsage {
                input: input_tokens,
                output: output_tokens,
            }),
            cost_estimate: Some(estimate_cost(
                "default",
                input_tokens * 4,
                output_tokens * 4,
            )),
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
    ) -> anyhow::Result<(String, i32, f64)> {
        let runner = SessionRunner::new();

        // Use 'opencode run' for non-interactive execution if possible.
        let mut args = vec!["run"];
        if let Some(model) = model {
            args.push("--model");
            args.push(model);
        }
        args.push(&scenario.task.prompt);

        let input_chars = scenario.task.prompt.len();
        let model_name = model.unwrap_or("default");

        let (output, exit_code) = runner.run_command("opencode", &args, cwd, timeout_secs)?;
        let output_chars = output.len();
        let cost = estimate_cost(model_name, input_chars, output_chars);

        Ok((output, exit_code, cost))
    }
}
