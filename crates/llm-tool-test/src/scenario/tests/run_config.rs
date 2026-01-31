use super::super::*;

#[test]
fn test_run_config() {
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
    assert!(scenario.run.is_none());
}

#[test]
fn test_run_config_partial() {
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
