pub mod types;

pub use types::*;

use std::path::Path;

pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Scenario> {
    let content = std::fs::read_to_string(path)?;
    let scenario: Scenario = serde_yaml::from_str(&content)?;
    Ok(scenario)
}

#[cfg(test)]
mod tests;
