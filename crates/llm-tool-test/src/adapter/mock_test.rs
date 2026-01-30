#[cfg(test)]
mod tests {
    use crate::adapter::mock::MockAdapter;
    use crate::adapter::ToolAdapter;
    use crate::scenario::Scenario;

    #[test]
    fn test_check_availability_always_succeeds() {
        let adapter = MockAdapter;
        assert!(adapter.check_availability().is_ok());
    }

    #[test]
    fn test_generate_transcript_min_notes() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 3
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qipu create"));
        assert!(transcript.lines().count() >= 3);
    }

    #[test]
    fn test_generate_transcript_note_exists() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: note_exists
      id: "qp-1234"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qp-1234"));
        assert!(transcript.contains("qipu create"));
    }

    #[test]
    fn test_generate_transcript_link_exists() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: link_exists
      from: "qp-1234"
      to: "qp-5678"
      link_type: "related"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qipu link add"));
        assert!(transcript.contains("--type related"));
        assert!(transcript.contains("qp-1234"));
        assert!(transcript.contains("qp-5678"));
    }

    #[test]
    fn test_generate_transcript_search_hit() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: search_hit
      query: "test query"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qipu create"));
        assert!(transcript.contains("test query"));
    }

    #[test]
    fn test_generate_transcript_tag_exists() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: tag_exists
      tag: "important"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qipu create"));
        assert!(transcript.contains("--tag important"));
    }

    #[test]
    fn test_generate_transcript_content_contains() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: content_contains
      id: "qp-1234"
      substring: "specific content"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qp-1234"));
        assert!(transcript.contains("specific content"));
    }

    #[test]
    fn test_generate_transcript_command_succeeds() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: command_succeeds
      command: "list"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert!(transcript.contains("qipu list"));
    }

    #[test]
    fn test_generate_transcript_empty_gates() {
        let adapter = MockAdapter;
        let scenario_yaml = r#"
name: test
description: "Test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates: []
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let transcript = adapter.generate_transcript(&scenario);

        assert_eq!(transcript.trim(), "qipu list");
    }

    #[test]
    fn test_end_to_end_scenario_execution() {
        let adapter = MockAdapter;

        let scenario_yaml = r#"
name: e2e_test
description: "End-to-end test scenario"
template_folder: qipu
task:
  prompt: "Create notes and links for testing"
evaluation:
  gates:
    - type: note_exists
      id: "qp-test-1"
    - type: note_exists
      id: "qp-test-2"
    - type: link_exists
      from: "qp-test-1"
      to: "qp-test-2"
      link_type: "related"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();

        let result = adapter.run(&scenario, temp_dir.path(), Some("mock"), 30);

        match result {
            Ok((output, exit_code, _cost, _token_usage)) => {
                assert_eq!(exit_code, 0, "Exit code should be 0");
                assert!(output.contains("qipu init"), "Should initialize qipu store");
                assert!(output.contains("qipu create"), "Should create notes");
                assert!(
                    output.contains("qp-test-1"),
                    "Should create note with specific ID"
                );
                assert!(output.contains("qipu link"), "Should create links");

                let db_path = temp_dir.path().join(".qipu/qipu.db");
                assert!(db_path.exists(), "Qipu database should be created");
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("doesn't exist on the filesystem")
                    || err_str.contains("No such file or directory")
                {
                    println!("Skipping test: qipu binary not found in PATH");
                    println!("To run this test: PATH=$PATH:./target/debug cargo test -p llm-tool-test test_end_to_end_scenario_execution");
                } else {
                    panic!("Scenario execution failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_end_to_end_with_search_and_tags() {
        let adapter = MockAdapter;

        let scenario_yaml = r#"
name: e2e_tags_test
description: "Test search and tag gates"
template_folder: qipu
task:
  prompt: "Create tagged notes for search testing"
evaluation:
  gates:
    - type: search_hit
      query: "important keyword"
    - type: tag_exists
      tag: "important"
    - type: content_contains
      id: "qp-content-1"
      substring: "specific content text"
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();

        let result = adapter.run(&scenario, temp_dir.path(), Some("mock"), 30);

        match result {
            Ok((output, exit_code, _cost, _token_usage)) => {
                assert_eq!(exit_code, 0);
                assert!(
                    output.contains("important keyword"),
                    "Should create note for search gate"
                );
                assert!(
                    output.contains("--tag important"),
                    "Should create tagged note"
                );
                assert!(
                    output.contains("qp-content-1"),
                    "Should create note with specific ID"
                );
                assert!(
                    output.contains("specific content text"),
                    "Should include content substring"
                );
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("doesn't exist on the filesystem")
                    || err_str.contains("No such file or directory")
                {
                    println!("Skipping test: qipu binary not found in PATH");
                    println!("To run this test: PATH=$PATH:./target/debug cargo test -p llm-tool-test test_end_to_end_with_search_and_tags");
                } else {
                    panic!("Scenario execution failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_end_to_end_command_succeeds_gate() {
        let adapter = MockAdapter;

        let scenario_yaml = r#"
name: e2e_command_test
description: "Test command succeeds gate"
template_folder: qipu
task:
  prompt: "Run various commands"
evaluation:
  gates:
    - type: command_succeeds
      command: "list"
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();

        let result = adapter.run(&scenario, temp_dir.path(), Some("mock"), 30);

        match result {
            Ok((output, exit_code, _cost, _token_usage)) => {
                assert_eq!(exit_code, 0);
                assert!(output.contains("qipu list"), "Should execute list command");
                assert!(output.contains("qipu create"), "Should create a note");
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("doesn't exist on the filesystem")
                    || err_str.contains("No such file or directory")
                {
                    println!("Skipping test: qipu binary not found in PATH");
                    println!("To run this test: PATH=$PATH:./target/debug cargo test -p llm-tool-test test_end_to_end_command_succeeds_gate");
                } else {
                    panic!("Scenario execution failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_end_to_end_cost_estimation() {
        let adapter = MockAdapter;

        let scenario_yaml = r#"
name: e2e_cost_test
description: "Test cost estimation"
template_folder: qipu
task:
  prompt: "Test cost calculation"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();

        let result = adapter.run(&scenario, temp_dir.path(), Some("mock"), 30);

        match result {
            Ok((_output, _exit_code, cost_opt, token_usage_opt)) => {
                assert!(
                    cost_opt.is_none(),
                    "Cost should be None when not reported by tool"
                );
                assert!(
                    token_usage_opt.is_none(),
                    "Token usage should be None when not reported by tool"
                );
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("doesn't exist on the filesystem")
                    || err_str.contains("No such file or directory")
                {
                    println!("Skipping test: qipu binary not found in PATH");
                    println!("To run this test: PATH=$PATH:./target/debug cargo test -p llm-tool-test test_end_to_end_cost_estimation");
                } else {
                    panic!("Scenario execution failed: {}", e);
                }
            }
        }
    }
}
