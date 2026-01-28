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
fixture: qipu
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
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: test_scenario
description: "A test scenario"
tier: 0
tags:
  - test
fixture: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("test_scenario.yaml"), scenario_content).unwrap();

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
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario1_content = r#"
name: scenario1
description: "First scenario"
tier: 0
tags:
  - smoke
fixture: qipu
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
fixture: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;

    fs::write(qipu_dir.join("scenario1.yaml"), scenario1_content).unwrap();
    fs::write(qipu_dir.join("scenario2.yaml"), scenario2_content).unwrap();

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
fn test_scenarios_command_with_pending_review() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["scenarios", "--pending-review"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No runs pending review"));
}

#[test]
fn test_report_command_no_runs() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["report"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No test runs found"));
}

#[test]
fn test_show_command_nonexistent_run() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["show", "nonexistent-run-id"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run not found"));
}

#[test]
fn test_compare_command_requires_two_ids() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["compare", "run1"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires exactly 2 run IDs"));
}

#[test]
fn test_clean_command_clears_cache() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["clean"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache cleared"));
}

#[test]
fn test_baseline_list_command() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["baseline", "list"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No baselines configured"));
}

#[test]
fn test_review_command_nonexistent_run() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["review", "nonexistent", "--dimension", "accuracy=0.9"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Run not found"));
}

#[test]
fn test_review_command_invalid_dimension_format() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["review", "some-id", "--dimension", "invalid-format"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid key=value pair"));
}

#[test]
fn test_review_command_dimension_score_too_high() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["review", "some-id", "--dimension", "accuracy=1.5"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be between 0.0 and 1.0"));
}

#[test]
fn test_review_command_dimension_score_negative() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["review", "some-id", "--dimension", "accuracy=-0.1"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be between 0.0 and 1.0"));
}

#[test]
fn test_baseline_clear_command() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["baseline", "clear", "test-scenario", "opencode"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Baseline cleared"));
}

#[test]
fn test_baseline_set_command_nonexistent_run() {
    let dir = tempdir().unwrap();
    llm_tool_test()
        .current_dir(dir.path())
        .args(["baseline", "set", "nonexistent-run-id"])
        .env("LLM_TOOL_TEST_ENABLED", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Run not found"));
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fixture: qipu
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
fn test_run_command_with_budget() {
    let dir = tempdir().unwrap();

    let fixtures_dir = dir.path().join("fixtures");
    let qipu_dir = fixtures_dir.join("qipu");
    fs::create_dir_all(&qipu_dir).unwrap();

    let scenario_content = r#"
name: budget_test
description: "Budget test"
fixture: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
    fs::write(qipu_dir.join("budget_test.yaml"), scenario_content).unwrap();

    llm_tool_test()
        .current_dir(dir.path())
        .args([
            "run",
            "--scenario",
            "fixtures/qipu/budget_test.yaml",
            "--max-usd",
            "0.10",
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
fixture: qipu
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
fixture: qipu
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
