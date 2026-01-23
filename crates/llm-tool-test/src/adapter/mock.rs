use super::ToolAdapter;
use crate::results::estimate_cost;
use crate::scenario::{Gate, Scenario};
use crate::session::SessionRunner;
use std::path::Path;

pub struct MockAdapter;

impl MockAdapter {
    /// Internal helper to run commands with event logging
    #[allow(dead_code)]
    pub fn run_with_events(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
        transcript_writer: &crate::transcript::TranscriptWriter,
    ) -> anyhow::Result<(String, i32, f64)> {
        use std::time::Instant;

        let start = Instant::now();
        let runner = SessionRunner::new();

        let transcript = self.generate_transcript(scenario);
        let mut full_output = String::new();

        let commands: Vec<&str> = transcript.lines().collect();
        let mut exit_code = 0;

        // Log spawn event for initialization
        transcript_writer.log_spawn("qipu", &["init".to_string()])?;
        let (init_out, init_code) = runner.run_command("qipu", &["init"], cwd, timeout_secs)?;
        transcript_writer.log_tool_result(&init_out, init_code)?;

        for (i, command) in commands.iter().enumerate() {
            let parts: Vec<String> = shlex::split(command).unwrap_or_default();
            if parts.is_empty() || !parts[0].starts_with("qipu") {
                continue;
            }

            let cmd_name = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

            // Log tool_call event
            transcript_writer.log_tool_call("bash", command)?;

            let (output, code) = runner.run_command(cmd_name, &args, cwd, timeout_secs)?;

            // Log tool_result event
            transcript_writer.log_tool_result(&output, code)?;

            if i > 0 {
                full_output.push_str("\n");
            }
            full_output.push_str(command);
            if !output.is_empty() {
                full_output.push_str("\n");
                full_output.push_str(&output);
                transcript_writer.log_output(&output)?;
            }

            if code != 0 && exit_code == 0 {
                exit_code = code;
            }
        }

        let duration_secs = start.elapsed().as_secs_f64();
        transcript_writer.log_complete(exit_code, duration_secs)?;

        let input_chars = scenario.task.prompt.len();
        let output_chars = full_output.len();
        let model_name = model.unwrap_or("mock");
        let cost = estimate_cost(model_name, input_chars, output_chars);

        Ok((full_output, exit_code, cost))
    }

    fn generate_transcript(&self, scenario: &Scenario) -> String {
        let mut commands = Vec::new();

        for gate in &scenario.evaluation.gates {
            match gate {
                Gate::MinNotes { count } => {
                    for i in 0..*count {
                        commands.push(format!("qipu create 'Mock Note {}'", i + 1));
                    }
                }
                Gate::NoteExists { id } => {
                    commands.push(format!("qipu create --id {} 'Note {}'", id, id));
                }
                Gate::LinkExists {
                    from,
                    to,
                    link_type,
                } => {
                    commands.push(format!(
                        "qipu link add --type {} {} {}",
                        link_type, from, to
                    ));
                }
                Gate::SearchHit { query } => {
                    commands.push(format!("qipu create 'Search Result - {}'", query));
                }
                Gate::TagExists { tag } => {
                    commands.push(format!("qipu create --tag {} 'Tagged Note'", tag));
                }
                Gate::ContentContains { id, substring } => {
                    commands.push(format!(
                        "qipu create --id {} 'Content Note - {}'",
                        id, substring
                    ));
                }
                Gate::CommandSucceeds { command } => {
                    commands.push(format!("qipu {}", command));
                }
                Gate::MinLinks { count } => {
                    // Create notes with links to satisfy the minimum link count
                    // Strategy: Create count+1 notes, where each note (except the first)
                    // links to the previous note, resulting in exactly 'count' links
                    for i in 0..=*count {
                        if i == 0 {
                            commands.push(format!(
                                "qipu create --id mock-link-{} 'Link Node {}'",
                                i, i
                            ));
                        } else {
                            commands.push(format!(
                                "qipu create --id mock-link-{} 'Link Node {}'",
                                i, i
                            ));
                        }
                    }
                    // Create links between notes
                    for i in 1..=*count {
                        commands.push(format!(
                            "qipu link add --type related mock-link-{} mock-link-{}",
                            i,
                            i - 1
                        ));
                    }
                }
                Gate::DoctorPasses => {
                    // Doctor check is automatic, no specific command needed
                }
                Gate::NoTranscriptErrors => {
                    // Transcript error checking is automatic, no specific command needed
                }
            }
        }

        if commands.is_empty() {
            commands.push("qipu list".to_string());
        }

        commands.join("\n")
    }
}

impl ToolAdapter for MockAdapter {
    fn name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        Ok(super::ToolStatus {
            available: true,
            version: Some("1.0.0-mock".to_string()),
            authenticated: true,
            budget_remaining: Some(100.0),
        })
    }

    fn execute_task(
        &self,
        context: &super::TaskContext,
        _work_dir: &Path,
        _transcript_dir: &Path,
    ) -> Result<super::ExecutionResult, super::AdapterError> {
        use std::time::Instant;

        let start = Instant::now();

        // For now, just return a mock result
        // In the future, this could parse context.task_prompt and generate appropriate commands
        let duration = start.elapsed();

        Ok(super::ExecutionResult {
            exit_code: 0,
            duration,
            token_usage: Some(super::TokenUsage {
                input: context.system_prompt.len() + context.task_prompt.len(),
                output: 100,
            }),
            cost_estimate: Some(0.001),
        })
    }

    fn estimate_cost(&self, prompt_tokens: usize) -> Option<super::CostEstimate> {
        // Mock adapter has very low cost
        Some(super::CostEstimate {
            estimated_usd: (prompt_tokens as f64) * 0.000001,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, f64)> {
        let runner = SessionRunner::new();

        let transcript = self.generate_transcript(scenario);
        let mut full_output = String::new();

        let commands: Vec<&str> = transcript.lines().collect();
        let mut exit_code = 0;

        let (init_output, init_code) = runner.run_command("qipu", &["init"], cwd, timeout_secs)?;
        full_output.push_str("qipu init");
        if !init_output.is_empty() {
            full_output.push_str("\n");
            full_output.push_str(&init_output);
        }
        if init_code != 0 && exit_code == 0 {
            exit_code = init_code;
        }

        for (i, command) in commands.iter().enumerate() {
            let parts: Vec<String> = shlex::split(command).unwrap_or_default();
            if parts.is_empty() || !parts[0].starts_with("qipu") {
                continue;
            }

            let cmd_name = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

            let (output, code) = runner.run_command(cmd_name, &args, cwd, timeout_secs)?;

            if i > 0 {
                full_output.push_str("\n");
            }
            full_output.push_str(command);
            if !output.is_empty() {
                full_output.push_str("\n");
                full_output.push_str(&output);
            }

            if code != 0 && exit_code == 0 {
                exit_code = code;
            }
        }

        let input_chars = scenario.task.prompt.len();
        let output_chars = full_output.len();
        let model_name = model.unwrap_or("mock");
        let cost = estimate_cost(model_name, input_chars, output_chars);

        Ok((full_output, exit_code, cost))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
            Ok((output, exit_code, _cost)) => {
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
                    return;
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
fixture: qipu
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
            Ok((output, exit_code, _cost)) => {
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
                    return;
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
fixture: qipu
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
            Ok((output, exit_code, _cost)) => {
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
                    return;
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
fixture: qipu
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
            Ok((_output, _exit_code, cost)) => {
                assert!(cost >= 0.0, "Cost should be non-negative");
                assert!(cost < 1.0, "Mock adapter cost should be very low");
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("doesn't exist on the filesystem")
                    || err_str.contains("No such file or directory")
                {
                    println!("Skipping test: qipu binary not found in PATH");
                    println!("To run this test: PATH=$PATH:./target/debug cargo test -p llm-tool-test test_end_to_end_cost_estimation");
                    return;
                } else {
                    panic!("Scenario execution failed: {}", e);
                }
            }
        }
    }
}
