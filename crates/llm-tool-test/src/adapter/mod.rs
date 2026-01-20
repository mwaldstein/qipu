pub mod amp;
pub mod claude_code;
pub mod mock;
pub mod opencode;

use crate::scenario::Scenario;
use std::path::Path;

pub trait ToolAdapter {
    /// Check if the tool is available and ready to use.
    fn check_availability(&self) -> anyhow::Result<()>;

    /// Run the tool with the given scenario in the specified working directory.
    /// Returns the tool output, exit code, and estimated cost in USD.
    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, f64)>;
}
