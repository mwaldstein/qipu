use super::ToolAdapter;
use crate::results::estimate_cost;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::path::Path;

pub struct OpenCodeAdapter;

impl ToolAdapter for OpenCodeAdapter {
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
