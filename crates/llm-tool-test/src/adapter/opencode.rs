use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use std::path::Path;

pub struct OpenCodeAdapter;

impl ToolAdapter for OpenCodeAdapter {
    fn run(&self, scenario: &Scenario, cwd: &Path) -> anyhow::Result<String> {
        let runner = SessionRunner::new();
        // We probably want to verify if opencode is installed first.
        // For now, we try to run it.

        // This is a placeholder for the actual invocation flags
        runner.run_command("opencode", &[&scenario.task.prompt], cwd)
    }
}
