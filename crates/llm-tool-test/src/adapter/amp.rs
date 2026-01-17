use super::ToolAdapter;
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
        match runner.run_command("amp", &["--version"], Path::new(".")) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(
                "Amp tool not found or failed to run: {}",
                e
            )),
        }
    }

    fn run(&self, scenario: &Scenario, cwd: &Path) -> anyhow::Result<String> {
        let runner = SessionRunner::new();

        // 1. Prepare prompt file
        let prompt_path = cwd.join("prompt.txt");
        fs::write(&prompt_path, &scenario.task.prompt)?;

        // 2. Prepare context (AGENTS.md is already in the fixture/cwd)
        // We assume AGENTS.md is the context.

        // 3. Construct command
        // Hypothesis: amp run --context AGENTS.md --prompt-file prompt.txt
        // Or similar. Adjusting to a likely CLI pattern.
        let args = [
            "run",
            "--context",
            "AGENTS.md",
            "--prompt-file",
            "prompt.txt",
        ];

        // 4. Run command
        runner.run_command("amp", &args, cwd)
    }
}
