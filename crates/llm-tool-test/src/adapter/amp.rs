use super::ToolAdapter;
use crate::results::estimate_cost;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

pub struct AmpAdapter;

impl ToolAdapter for AmpAdapter {
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
