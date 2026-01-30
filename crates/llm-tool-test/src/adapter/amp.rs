use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

// NOTE: Integration with 'amp' is currently de-prioritized as we don't have a subscription to test it with.
// This adapter is kept for reference but may not be fully functional or tested.
pub struct AmpAdapter;

impl ToolAdapter for AmpAdapter {
    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("amp", &["--version"], Path::new("."), 10) {
            Ok(_) => {
                Ok(super::ToolStatus {
                    available: true,
                    authenticated: true, // amp doesn't require auth
                })
            }
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "Amp tool not found or failed to run: {}",
                e
            ))),
        }
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
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<super::TokenUsage>)> {
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

        let (output, exit_code) = runner.run_command("amp", &args, cwd, timeout_secs)?;

        Ok((output, exit_code, None, None))
    }
}
