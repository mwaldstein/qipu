use super::super::*;

#[test]
fn test_note_exists_gate() {
    let yaml = r#"
name: test
description: "Test"
template_folder: qipu
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
template_folder: qipu
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
template_folder: qipu
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
template_folder: qipu
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
template_folder: qipu
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
fn test_doctor_passes_gate() {
    let yaml = r#"
name: test
description: "Test"
template_folder: qipu
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
template_folder: qipu
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
