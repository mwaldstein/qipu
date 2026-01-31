use super::super::*;

#[test]
fn test_load_scenario_with_setup() {
    let yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
setup:
  commands:
    - "qipu init"
    - "echo setup complete"
"#;
    let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(scenario.name, "test");
    assert!(scenario.setup.is_some());
    let setup = scenario.setup.unwrap();
    assert_eq!(setup.commands.len(), 2);
    assert_eq!(setup.commands[0], "qipu init");
    assert_eq!(setup.commands[1], "echo setup complete");
}

#[test]
fn test_load_scenario_without_setup() {
    let yaml = r#"
name: test
description: "Test"
template_folder: qipu
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
fn test_setup_commands() {
    let yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test prompt"
evaluation:
  gates:
    - type: min_notes
      count: 1
setup:
  commands:
    - "pwd"
    - "ls -la"
"#;
    let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
    assert!(scenario.setup.is_some());
    let setup = scenario.setup.unwrap();
    assert_eq!(setup.commands.len(), 2);
    assert_eq!(setup.commands[0], "pwd");
    assert_eq!(setup.commands[1], "ls -la");
}
