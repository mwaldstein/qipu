use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::path::Path;

pub struct OpenCodeAdapter;

impl ToolAdapter for OpenCodeAdapter {
    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        match runner.run_command("opencode", &["--version"], Path::new(".")) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("OpenCode tool not found: {}", e)),
        }
    }

    fn run(&self, scenario: &Scenario, cwd: &Path) -> anyhow::Result<String> {
        let runner = SessionRunner::new();

        // Use 'opencode run' for non-interactive execution if possible.
        let args = ["run", &scenario.task.prompt];

        runner.run_command("opencode", &args, cwd)
    }
}
