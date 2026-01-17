use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub fixture: String,
    pub task: Task,
    pub evaluation: Evaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub gates: Vec<Gate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Gate {
    MinNotes { count: usize },
    MinLinks { count: usize },
    SearchHit { query: String },
}

pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Scenario> {
    let content = std::fs::read_to_string(path)?;
    let scenario: Scenario = serde_yaml::from_str(&content)?;
    Ok(scenario)
}
