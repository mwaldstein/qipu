use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::fs;
use std::path::Path;

pub struct ClaudeCodeAdapter;

impl ToolAdapter for ClaudeCodeAdapter {
    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        match runner.run_command("claude", &["--version"], Path::new(".")) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Claude Code tool not found: {}", e)),
        }
    }

    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
    ) -> anyhow::Result<(String, i32)> {
        let runner = SessionRunner::new();

        let mut args = vec!["run"];
        if let Some(model) = model {
            args.push("--model");
            args.push(model);
        }

        let prompt_path = cwd.join("prompt.txt");
        fs::write(&prompt_path, &scenario.task.prompt)?;

        args.push("--prompt-file");
        args.push("prompt.txt");

        runner.run_command("claude", &args, cwd)
    }
}
