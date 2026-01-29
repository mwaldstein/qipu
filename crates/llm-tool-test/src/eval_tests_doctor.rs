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

fn create_note_with_stdin(env_root: &std::path::Path, content: &str) {
    let qipu = crate::eval_helpers::get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");

    let mut child = std::process::Command::new(qipu_abs)
        .arg("capture")
        .current_dir(env_root)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(content.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait");
    assert!(
        output.status.success(),
        "Capture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_command_doctor_transcript_gates() {
    let (_dir, env_root) = setup_env();

    create_note_with_stdin(&env_root, "This is a test note #test");

    let command_scenario_pass = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::CommandSucceeds {
                command: "list".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&command_scenario_pass, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let command_scenario_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::CommandSucceeds {
                command: "nonexistent-command".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&command_scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);

    create_note_with_stdin(&env_root, "Test note for doctor check");

    let doctor_scenario = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::DoctorPasses],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&doctor_scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);

    let json = crate::eval_helpers::run_qipu_json(&["list"], &env_root).unwrap();
    let first_note_path = json
        .get(0)
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .expect("No path found");

    let note_path = env_root.join(first_note_path);
    std::fs::remove_file(&note_path).expect("Failed to delete note file");

    let metrics = evaluate(&doctor_scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);

    let artifacts_dir = env_root.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir).unwrap();

    let transcript_no_errors = "qipu create --title 'Test'\nqp-abc123\nqipu list\n...";
    std::fs::write(
        artifacts_dir.join("transcript.raw.txt"),
        transcript_no_errors,
    )
    .unwrap();

    let transcript_scenario = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::NoTranscriptErrors],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };

    let metrics = evaluate(&transcript_scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);

    let transcript_with_errors = "qipu create --title 'Test'\nError: invalid input\nExit code: 1\nqipu create --title 'Test 2'\nqp-abc123";
    std::fs::write(
        artifacts_dir.join("transcript.raw.txt"),
        transcript_with_errors,
    )
    .unwrap();

    let metrics = evaluate(&transcript_scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);

    std::fs::remove_file(artifacts_dir.join("transcript.raw.txt")).unwrap();

    let metrics = evaluate(&transcript_scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);
}
