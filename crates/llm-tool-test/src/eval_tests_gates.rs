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
    }
}

fn get_first_note_id(env_root: &std::path::Path) -> String {
    let json = crate::eval_helpers::run_qipu_json(&["list"], env_root).unwrap();
    json.get(0)
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .expect("No notes found")
        .to_string()
}

fn get_second_note_id(env_root: &std::path::Path, first_note_id: &str) -> String {
    let json = crate::eval_helpers::run_qipu_json(&["list"], env_root).unwrap();
    json.as_array()
        .and_then(|arr| {
            arr.iter().find_map(|v| {
                let id = v.get("id").and_then(|v| v.as_str());
                if id != Some(first_note_id) {
                    id
                } else {
                    None
                }
            })
        })
        .expect("Second note not found")
        .to_string()
}

fn run_qipu_command(env_root: &std::path::Path, args: &[&str]) {
    let qipu = crate::eval_helpers::get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");
    let output = std::process::Command::new(qipu_abs)
        .args(args)
        .current_dir(env_root)
        .output()
        .expect("Failed to run qipu command");
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_min_notes_gate_fail() {
    let (_dir, env_root) = setup_env();

    let scenario = create_test_scenario(Gate::MinNotes { count: 1 });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);
}

#[test]
fn test_min_notes_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::MinNotes { count: 1 });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);
}

#[test]
fn test_min_notes_gate_fail_with_one_note() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::MinNotes { count: 2 });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_search_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::SearchHit {
        query: "test".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_search_gate_fail() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::SearchHit {
        query: "nonexistent".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_note_exists_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let note_id = get_first_note_id(&env_root);
    let scenario = create_test_scenario(Gate::NoteExists { id: note_id });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_note_exists_gate_fail() {
    let (_dir, env_root) = setup_env();

    let scenario = create_test_scenario(Gate::NoteExists {
        id: "qp-nonexistent".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_link_exists_gate_fail() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let note_id = get_first_note_id(&env_root);
    let scenario = create_test_scenario(Gate::LinkExists {
        from: note_id.clone(),
        to: note_id,
        link_type: "related".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_link_exists_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "First note for link test");
    create_note_with_stdin(&env_root, "Second note for link test");

    let first_note_id = get_first_note_id(&env_root);
    let second_note_id = get_second_note_id(&env_root, &first_note_id);

    run_qipu_command(
        &env_root,
        &[
            "link",
            "add",
            &first_note_id,
            &second_note_id,
            "--type",
            "related",
        ],
    );

    let scenario = create_test_scenario(Gate::LinkExists {
        from: first_note_id,
        to: second_note_id,
        link_type: "related".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_tag_exists_gate_fail() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario = create_test_scenario(Gate::TagExists {
        tag: "nonexistent".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_tag_exists_gate_pass() {
    let (_dir, env_root) = setup_env();

    run_qipu_command(
        &env_root,
        &["create", "Important note", "--tag", "important"],
    );

    let scenario = create_test_scenario(Gate::TagExists {
        tag: "important".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_content_contains_gate_pass() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let note_id = get_first_note_id(&env_root);
    let scenario = create_test_scenario(Gate::ContentContains {
        id: note_id,
        substring: "test note".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
}

#[test]
fn test_content_contains_gate_fail() {
    let (_dir, env_root) = setup_env();
    create_note_with_stdin(&env_root, "This is a test note #test");

    let note_id = get_first_note_id(&env_root);
    let scenario = create_test_scenario(Gate::ContentContains {
        id: note_id,
        substring: "nonexistent".to_string(),
    });
    let metrics = evaluate(&scenario, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}
