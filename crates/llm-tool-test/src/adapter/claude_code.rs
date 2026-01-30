use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

pub struct ClaudeCodeAdapter;

impl ToolAdapter for ClaudeCodeAdapter {
    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("claude", &["--version"], Path::new("."), 10) {
            Ok(_) => Ok(super::ToolStatus {
                available: true,
                authenticated: true,
            }),
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "Claude Code tool not found: {}",
                e
            ))),
        }
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
