use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

pub struct ClaudeCodeAdapter;

impl ToolAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("claude", &["--version"], Path::new("."), 10) {
            Ok((output, _)) => {
                let version = output.trim().to_string();
                Ok(super::ToolStatus {
                    available: true,
                    version: Some(version),
                    authenticated: true,
                    budget_remaining: None,
                })
            }
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "Claude Code tool not found: {}",
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
        use std::time::Instant;

        let start = Instant::now();
        let runner = SessionRunner::new();

        // Write the full prompt (system + task) to a file
        let full_prompt = format!("{}\n\n{}", context.system_prompt, context.task_prompt);
        let prompt_path = work_dir.join("prompt.txt");
        fs::write(&prompt_path, &full_prompt).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write prompt: {}", e))
        })?;

        let args = vec!["run", "--prompt-file", "prompt.txt"];
        let timeout_secs = context.timeout.as_secs();

        let (output, exit_code) = runner
            .run_command("claude", &args, work_dir, timeout_secs)
            .map_err(|e| {
                super::AdapterError::ExecutionFailed(format!("claude execution failed: {}", e))
            })?;

        // Write transcript
        let transcript_path = transcript_dir.join("transcript.raw.txt");
        fs::write(&transcript_path, &output).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write transcript: {}", e))
        })?;

        let duration = start.elapsed();

        Ok(super::ExecutionResult {
            exit_code,
            duration,
            token_usage: None,
            cost_estimate: None,
        })
    }

    fn estimate_cost(&self, prompt_tokens: usize) -> Option<super::CostEstimate> {
        // Estimate based on typical Claude pricing
        let input_cost = (prompt_tokens as f64) / 1_000_000.0 * 3.0; // $3/M tokens
        let output_cost = (prompt_tokens as f64 * 0.5) / 1_000_000.0 * 15.0; // $15/M tokens
        Some(super::CostEstimate {
            estimated_usd: input_cost + output_cost,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        match runner.run_command("claude", &["--version"], Path::new("."), 10) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Claude Code tool not found: {}", e)),
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

        let mut args = vec!["run"];
        if let Some(model) = model {
            args.push("--model");
            args.push(model);
        }

        let prompt_path = cwd.join("prompt.txt");
        fs::write(&prompt_path, &scenario.task.prompt)?;

        let (output, exit_code) = runner.run_command("claude", &args, cwd, timeout_secs)?;

        Ok((output, exit_code, None, None))
    }
}
