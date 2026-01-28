mod support;

use crate::support::llm_tool_test;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    llm_tool_test()
        .arg("--help")
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_cli_version() {
    llm_tool_test()
        .arg("--version")
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-tool-test"));
}

#[test]
fn test_run_command_requires_env_var() {
    let dir = tempdir().unwrap();
    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: test_basic
description: "Basic test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("test_basic.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["run", "--scenario", "fixtures/qipu/test_basic.yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("LLM_TOOL_TEST_ENABLED"));
}

#[test]
fn test_run_command_with_all_flag_requires_scenarios() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["run", "--all"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_scenarios_command_no_fixtures() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["scenarios"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available scenarios"));
}

#[test]
fn test_scenarios_command_with_fixtures() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    let scenario_content = r#"
name: test_scenario
description: "A test scenario"
tier: 0
tags:
  - test
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(fixtures_dir.join("test_scenario.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["scenarios"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("test_scenario"))
        .stdout(predicate::str::contains("A test scenario"));
}

#[test]
fn test_scenarios_command_with_tags_filter() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    let scenario1_content = r#"
name: scenario1
description: "First scenario"
tier: 0
tags:
  - smoke
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    let scenario2_content = r#"
name: scenario2
description: "Second scenario"
tier: 0
tags:
  - integration
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;

    fs::write(fixtures_dir.join("scenario1.yaml"), scenario1_content).unwrap();
    fs::write(fixtures_dir.join("scenario2.yaml"), scenario2_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["scenarios", "--tags", "smoke"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("scenario1"))
        .stdout(predicate::str::contains("[smoke]"))
        .stdout(predicate::str::contains("scenario2").not());
}

#[test]
fn test_run_command_dry_run() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: dry_run_test
description: "Dry run test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("dry_run_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/dry_run_test.yaml",
            "--dry-run",
        ])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_tags() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario1_content = r#"
name: tagged_scenario
description: "Tagged scenario"
tier: 0
tags:
  - smoke
  - quick
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    let scenario2_content = r#"
name: untagged_scenario
description: "Untagged scenario"
tier: 0
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;

    fs::write(qipu_dir.join("tagged_scenario.yaml"), scenario1_content).unwrap();
    fs::write(qipu_dir.join("untagged_scenario.yaml"), scenario2_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["run", "--all", "--tags", "smoke"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_tool_option() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: tool_test
description: "Tool option test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("tool_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/tool_test.yaml",
            "--tool",
            "mock",
        ])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_model_option() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: model_test
description: "Model option test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("model_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/model_test.yaml",
            "--model",
            "test-model",
        ])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_tier_filter() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario1_content = r#"
name: tier0_scenario
description: "Tier 0 scenario"
tier: 0
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    let scenario2_content = r#"
name: tier1_scenario
description: "Tier 1 scenario"
tier: 1
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;

    fs::write(qipu_dir.join("tier0_scenario.yaml"), scenario1_content).unwrap();
    fs::write(qipu_dir.join("tier1_scenario.yaml"), scenario2_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["run", "--all", "--tier", "0"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_timeout() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: timeout_test
description: "Timeout test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("timeout_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/timeout_test.yaml",
            "--timeout-secs",
            "60",
        ])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_no_cache() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: no_cache_test
description: "No cache test"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("no_cache_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/no_cache_test.yaml",
            "--no-cache",
        ])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success();
}

#[test]
fn test_run_command_matrix_multiple_tools() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: matrix_test
description: "Matrix run test"
tool_matrix:
  - tool: mock
    models:
      - model1
      - model2
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("matrix_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args(["run", "--scenario", "fixtures/qipu/matrix_test.yaml"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Matrix run"));
}

#[test]
fn test_clean_command_with_older_than() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["clean", "--older-than", "7d"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache cleared"));
}

#[test]
fn test_clean_command_invalid_duration() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["clean", "--older-than", "invalid"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure();
}
