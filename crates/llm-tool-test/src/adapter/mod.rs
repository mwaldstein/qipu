pub mod opencode;

use crate::scenario::Scenario;
use std::path::Path;

pub trait ToolAdapter {
    fn run(&self, scenario: &Scenario, cwd: &Path) -> anyhow::Result<String>;
}
