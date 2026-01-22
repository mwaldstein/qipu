use super::ToolAdapter;
use crate::results::estimate_cost;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

pub struct AmpAdapter;

impl ToolAdapter for AmpAdapter {
    fn name(&self) -> &str {
        "amp"
    }

    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("amp", &["--version"], Path::new("."), 10) {
            Ok((output, _)) => {
                let version = output.trim().to_string();
                Ok(super::ToolStatus {
                    available: true,
                    version: Some(version),
                    authenticated: true, // amp doesn't require auth
                    budget_remaining: None,
                })
            }
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "Amp tool not found or failed to run: {}",
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
        let full_prompt = format!(
            "{}\n\n---\n\n{}",
            context.system_prompt, context.task_prompt
        );
        let prompt_path = work_dir.join("prompt.txt");
        fs::write(&prompt_path, &full_prompt).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write prompt: {}", e))
        })?;

        let prompt_arg = format!("@{}", prompt_path.display());
        let args = vec!["-x", &prompt_arg];

        let timeout_secs = context.timeout.as_secs();
        let (output, exit_code) = runner
            .run_command("amp", &args, work_dir, timeout_secs)
            .map_err(|e| {
                super::AdapterError::ExecutionFailed(format!("amp execution failed: {}", e))
            })?;

        // Write transcript
        let transcript_path = transcript_dir.join("transcript.raw.txt");
        fs::write(&transcript_path, &output).map_err(|e| {
            super::AdapterError::ExecutionFailed(format!("Failed to write transcript: {}", e))
        })?;

        let duration = start.elapsed();
        let input_tokens = (context.system_prompt.len() + context.task_prompt.len()) / 4; // rough estimate
        let output_tokens = output.len() / 4;

        Ok(super::ExecutionResult {
            exit_code,
            duration,
            token_usage: Some(super::TokenUsage {
                input: input_tokens,
                output: output_tokens,
            }),
            cost_estimate: Some(estimate_cost("smart", input_tokens * 4, output_tokens * 4)),
        })
    }

    fn estimate_cost(&self, prompt_tokens: usize) -> Option<super::CostEstimate> {
        // Estimate based on typical amp pricing (smart mode)
        let input_cost = (prompt_tokens as f64) / 1_000_000.0 * 3.0; // $3/M tokens
        let output_cost = (prompt_tokens as f64 * 0.5) / 1_000_000.0 * 15.0; // $15/M tokens, assume 0.5x output
        Some(super::CostEstimate {
            estimated_usd: input_cost + output_cost,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        // Check if 'amp' is in PATH by running 'amp --version' or similar.
        // Using 'which' or just trying to run it.
        match runner.run_command("amp", &["--version"], Path::new("."), 10) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(
                "Amp tool not found or failed to run: {}",
                e
            )),
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

        let prompt_path = cwd.join("prompt.txt");
        fs::write(&prompt_path, &scenario.task.prompt)?;

        let mut args = vec!["-x"];

        if let Some(model) = model {
            args.push("--mode");
            args.push(model);
        }

        let prompt_content = fs::read_to_string(&prompt_path)?;
        let full_prompt = if cwd.join("AGENTS.md").exists() {
            let agents_content = fs::read_to_string(cwd.join("AGENTS.md"))?;
            format!("{}\n\n---\n\n{}", agents_content, prompt_content)
        } else {
            prompt_content
        };

        let prompt_arg_path = cwd.join("prompt_arg.txt");
        fs::write(&prompt_arg_path, &full_prompt)?;

        let prompt_arg = format!("@{}", prompt_arg_path.display());
        args.push(&prompt_arg);

        let input_chars = full_prompt.len();
        let model_name = model.unwrap_or("smart");

        let (output, exit_code) = runner.run_command("amp", &args, cwd, timeout_secs)?;
        let output_chars = output.len();
        let cost = estimate_cost(model_name, input_chars, output_chars);

        Ok((output, exit_code, cost))
    }
}
