use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub fixture: String,
    pub task: Task,
    pub evaluation: Evaluation,
    #[serde(default = "default_tier")]
    pub tier: usize,
    #[serde(default)]
    pub tool_matrix: Option<Vec<ToolConfig>>,
    #[serde(default)]
    pub setup: Option<Vec<SetupStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_tier() -> usize {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool: String,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub gates: Vec<Gate>,
    #[serde(default)]
    pub judge: Option<JudgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    pub enabled: bool,
    pub rubric: String,
    pub pass_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Gate {
    MinNotes {
        count: usize,
    },
    MinLinks {
        count: usize,
    },
    SearchHit {
        query: String,
    },
    NoteExists {
        id: String,
    },
    LinkExists {
        from: String,
        to: String,
        link_type: String,
    },
}

pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Scenario> {
    let content = std::fs::read_to_string(path)?;
    let scenario: Scenario = serde_yaml::from_str(&content)?;
    Ok(scenario)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_scenario_without_tool_matrix() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.tool_matrix.is_none());
    }

    #[test]
    fn test_load_scenario_with_tool_matrix() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
tool_matrix:
  - tool: opencode
    models: [claude-sonnet-4-20250514, gpt-4o]
  - tool: amp
    models: [default]
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.tool_matrix.is_some());

        let matrix = scenario.tool_matrix.unwrap();
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].tool, "opencode");
        assert_eq!(matrix[0].models, vec!["claude-sonnet-4-20250514", "gpt-4o"]);
        assert_eq!(matrix[1].tool, "amp");
        assert_eq!(matrix[1].models, vec!["default"]);
    }

    #[test]
    fn test_load_scenario_with_empty_models() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
tool_matrix:
  - tool: opencode
    models: []
  - tool: amp
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.tool_matrix.is_some());

        let matrix = scenario.tool_matrix.unwrap();
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].tool, "opencode");
        assert!(matrix[0].models.is_empty());
        assert_eq!(matrix[1].tool, "amp");
        assert!(matrix[1].models.is_empty());
    }

    #[test]
    fn test_load_scenario_with_tier() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
tier: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.tier, 1);
        assert!(scenario.tool_matrix.is_none());
    }

    #[test]
    fn test_default_tier() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.tier, 0);
    }

    #[test]
    fn test_load_scenario_with_setup() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
setup:
  - command: "qipu"
    args: ["init"]
  - command: "echo"
    args: ["setup complete"]
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.setup.is_some());
        let setup = scenario.setup.unwrap();
        assert_eq!(setup.len(), 2);
        assert_eq!(setup[0].command, "qipu");
        assert_eq!(setup[0].args, vec!["init"]);
        assert_eq!(setup[1].command, "echo");
        assert_eq!(setup[1].args, vec!["setup complete"]);
    }

    #[test]
    fn test_load_scenario_without_setup() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.setup.is_none());
    }

    #[test]
    fn test_setup_step_with_no_args() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
setup:
  - command: "pwd"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert!(scenario.setup.is_some());
        let setup = scenario.setup.unwrap();
        assert_eq!(setup.len(), 1);
        assert_eq!(setup[0].command, "pwd");
        assert!(setup[0].args.is_empty());
    }

    #[test]
    fn test_note_exists_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: note_exists
      id: "qp-1234"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::NoteExists { id } => assert_eq!(id, "qp-1234"),
            _ => panic!("Expected NoteExists gate"),
        }
    }

    #[test]
    fn test_link_exists_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: link_exists
      from: "qp-1234"
      to: "qp-5678"
      link_type: "related"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::LinkExists {
                from,
                to,
                link_type,
            } => {
                assert_eq!(from, "qp-1234");
                assert_eq!(to, "qp-5678");
                assert_eq!(link_type, "related");
            }
            _ => panic!("Expected LinkExists gate"),
        }
    }
}
