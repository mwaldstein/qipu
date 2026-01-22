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
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub docs: Option<DocsConfig>,
    #[serde(default)]
    pub run: Option<RunConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    #[serde(default)]
    pub max_usd: Option<f64>,
    #[serde(default = "default_cache")]
    pub cache: bool,
}

fn default_cache() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    #[serde(default)]
    pub prime: bool,
    #[serde(default)]
    pub help_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub max_turns: Option<usize>,
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
    TagExists {
        tag: String,
    },
    ContentContains {
        id: String,
        substring: String,
    },
    CommandSucceeds {
        command: String,
    },
    DoctorPasses,
    NoTranscriptErrors,
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

    #[test]
    fn test_tag_exists_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: tag_exists
      tag: "important"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::TagExists { tag } => assert_eq!(tag, "important"),
            _ => panic!("Expected TagExists gate"),
        }
    }

    #[test]
    fn test_content_contains_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: content_contains
      id: "qp-1234"
      substring: "important keyword"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::ContentContains { id, substring } => {
                assert_eq!(id, "qp-1234");
                assert_eq!(substring, "important keyword");
            }
            _ => panic!("Expected ContentContains gate"),
        }
    }

    #[test]
    fn test_command_succeeds_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: command_succeeds
      command: "list"
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::CommandSucceeds { command } => assert_eq!(command, "list"),
            _ => panic!("Expected CommandSucceeds gate"),
        }
    }

    #[test]
    fn test_docs_config() {
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
docs:
  prime: true
  help_commands:
    - create
    - link
    - list
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.docs.is_some());
        let docs = scenario.docs.unwrap();
        assert!(docs.prime);
        assert_eq!(docs.help_commands, vec!["create", "link", "list"]);
    }

    #[test]
    fn test_docs_config_optional() {
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
        assert!(scenario.docs.is_none());
    }

    #[test]
    fn test_docs_config_defaults() {
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
docs:
  prime: false
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.docs.is_some());
        let docs = scenario.docs.unwrap();
        assert!(!docs.prime);
        assert!(docs.help_commands.is_empty());
    }

    #[test]
    fn test_run_config() {
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
run:
  timeout_secs: 600
  max_turns: 40
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.run.is_some());
        let run = scenario.run.unwrap();
        assert_eq!(run.timeout_secs, Some(600));
        assert_eq!(run.max_turns, Some(40));
    }

    #[test]
    fn test_run_config_optional() {
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
        assert!(scenario.run.is_none());
    }

    #[test]
    fn test_run_config_partial() {
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
run:
  timeout_secs: 300
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.run.is_some());
        let run = scenario.run.unwrap();
        assert_eq!(run.timeout_secs, Some(300));
        assert_eq!(run.max_turns, None);
    }

    #[test]
    fn test_cost_config() {
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
cost:
  max_usd: 0.75
  cache: false
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.cost.is_some());
        let cost = scenario.cost.unwrap();
        assert_eq!(cost.max_usd, Some(0.75));
        assert!(!cost.cache);
    }

    #[test]
    fn test_cost_config_defaults() {
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
cost:
  max_usd: 1.00
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert!(scenario.cost.is_some());
        let cost = scenario.cost.unwrap();
        assert_eq!(cost.max_usd, Some(1.00));
        assert!(cost.cache); // Default is true
    }

    #[test]
    fn test_tags_field() {
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
tags: [capture, links, retrieval]
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.tags, vec!["capture", "links", "retrieval"]);
    }

    #[test]
    fn test_complete_scenario() {
        let yaml = r#"
name: capture_article_basic
description: "Capture article ideas as linked notes"
fixture: qipu
tags: [capture, links, retrieval]
docs:
  prime: true
  help_commands:
    - create
    - link
    - list
    - search
    - show
task:
  prompt: "Capture key ideas from this article"
tool_matrix:
  - tool: amp
    models: [default]
  - tool: opencode
    models: [default]
run:
  timeout_secs: 600
  max_turns: 40
evaluation:
  gates:
    - type: min_notes
      count: 3
    - type: min_links
      count: 1
cost:
  max_usd: 0.75
  cache: true
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "capture_article_basic");
        assert_eq!(
            scenario.description,
            "Capture article ideas as linked notes"
        );
        assert_eq!(scenario.tags, vec!["capture", "links", "retrieval"]);

        // Docs
        assert!(scenario.docs.is_some());
        let docs = scenario.docs.unwrap();
        assert!(docs.prime);
        assert_eq!(docs.help_commands.len(), 5);

        // Run config
        assert!(scenario.run.is_some());
        let run = scenario.run.unwrap();
        assert_eq!(run.timeout_secs, Some(600));
        assert_eq!(run.max_turns, Some(40));

        // Cost
        assert!(scenario.cost.is_some());
        let cost = scenario.cost.unwrap();
        assert_eq!(cost.max_usd, Some(0.75));
        assert!(cost.cache);

        // Tool matrix
        assert!(scenario.tool_matrix.is_some());
        let matrix = scenario.tool_matrix.unwrap();
        assert_eq!(matrix.len(), 2);

        // Evaluation
        assert_eq!(scenario.evaluation.gates.len(), 2);
    }

    #[test]
    fn test_doctor_passes_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: doctor_passes
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::DoctorPasses => (),
            _ => panic!("Expected DoctorPasses gate"),
        }
    }

    #[test]
    fn test_no_transcript_errors_gate() {
        let yaml = r#"
name: test
description: "Test"
fixture: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: no_transcript_errors
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.evaluation.gates.len(), 1);
        match &scenario.evaluation.gates[0] {
            Gate::NoTranscriptErrors => (),
            _ => panic!("Expected NoTranscriptErrors gate"),
        }
    }
}
