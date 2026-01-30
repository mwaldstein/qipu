use crate::eval_helpers::*;
use crate::evaluation::*;
use crate::scenario::{Evaluation, Gate, Scenario, Task};
use tempfile::tempdir;

fn setup_env() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();

    let qipu = crate::eval_helpers::get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");

    let output = std::process::Command::new(qipu_abs)
        .arg("init")
        .current_dir(&path)
        .output()
        .expect("failed to run qipu init");

    assert!(output.status.success());

    (dir, path)
}

fn create_test_scenario(gate: Gate) -> Scenario {
    Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![gate],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
        cost: None,
    }
}

fn get_first_note_path(env_root: &std::path::Path) -> String {
    let json = crate::eval_helpers::run_qipu_json(&["list"], env_root).unwrap();
    json.get(0)
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .expect("No path found")
        .to_string()
}

#[test]
fn test_command_succeeds_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::CommandSucceeds {
        command: "list".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_command_succeeds_gate_fail() {
    let (_dir, env_root) = setup_env();

    let scenario = create_test_scenario(Gate::CommandSucceeds {
        command: "nonexistent-command".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_doctor_passes_gate() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "Test note for doctor check");

    let scenario = create_test_scenario(Gate::DoctorPasses);
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);
}

#[test]
fn test_doctor_passes_gate_fail_after_delete() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "Test note for doctor check");

    let scenario = create_test_scenario(Gate::DoctorPasses);
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let first_note_path = get_first_note_path(&env_root);
    let note_path = env_root.join(&first_note_path);
    std::fs::remove_file(&note_path).expect("Failed to delete note file");

    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);
}

#[test]
fn test_no_transcript_errors_gate_pass() {
    let (_dir, env_root) = setup_env();

    let artifacts_dir = env_root.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir).unwrap();

    let transcript_no_errors = "qipu create --title 'Test'\nqp-abc123\nqipu list\n...";
    std::fs::write(
        artifacts_dir.join("transcript.raw.txt"),
        transcript_no_errors,
    )
    .unwrap();

    let scenario = create_test_scenario(Gate::NoTranscriptErrors);
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);
}

#[test]
fn test_no_transcript_errors_gate_fail_with_errors() {
    let (_dir, env_root) = setup_env();

    let artifacts_dir = env_root.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir).unwrap();

    let transcript_with_errors = "qipu create --title 'Test'\nError: invalid input\nExit code: 1\nqipu create --title 'Test 2'\nqp-abc123";
    std::fs::write(
        artifacts_dir.join("transcript.raw.txt"),
        transcript_with_errors,
    )
    .unwrap();

    let scenario = create_test_scenario(Gate::NoTranscriptErrors);
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);
}

#[test]
fn test_no_transcript_errors_gate_fail_no_file() {
    let (_dir, env_root) = setup_env();

    let scenario = create_test_scenario(Gate::NoTranscriptErrors);
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);
}
