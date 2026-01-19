pub mod amp;
pub mod opencode;

use crate::scenario::Scenario;
use std::path::Path;

pub trait ToolAdapter {
    /// Check if the tool is available and ready to use.
    fn check_availability(&self) -> anyhow::Result<()>;

    /// Run the tool with the given scenario in the specified working directory.
    /// Returns the tool output and exit code.
    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
    ) -> anyhow::Result<(String, i32)>;
}
