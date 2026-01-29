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
fn test_min_notes_min_links_gates() {
    let (_dir, env_root) = setup_env();

    let scenario_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::MinNotes { count: 1 }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };

    let metrics = evaluate(&scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
    assert!(!metrics.details[0].passed);

    create_note_with_stdin(&env_root, "This is a test note #test");

    let metrics = evaluate(&scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);
    assert!(metrics.details[0].passed);

    let scenario_fail_2 = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::MinNotes { count: 2 }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&scenario_fail_2, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_search_gates() {
    let (_dir, env_root) = setup_env();

    create_note_with_stdin(&env_root, "This is a test note #test");

    let scenario_search = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::SearchHit {
                query: "test".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&scenario_search, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let scenario_search_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::SearchHit {
                query: "nonexistent".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&scenario_search_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}

#[test]
fn test_note_link_tag_content_gates() {
    let (_dir, env_root) = setup_env();

    create_note_with_stdin(&env_root, "This is a test note #test");

    let json = crate::eval_helpers::run_qipu_json(&["list"], &env_root).unwrap();
    let first_note_id = json
        .get(0)
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .expect("No notes found");

    let scenario_note_exists = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::NoteExists {
                id: first_note_id.to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&scenario_note_exists, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let scenario_note_exists_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::NoteExists {
                id: "qp-nonexistent".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&scenario_note_exists_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);

    let link_scenario_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::LinkExists {
                from: first_note_id.to_string(),
                to: first_note_id.to_string(),
                link_type: "related".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&link_scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);

    create_note_with_stdin(&env_root, "Second note for link test");
    let json = crate::eval_helpers::run_qipu_json(&["list"], &env_root).unwrap();
    let second_note_id = json
        .as_array()
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
        .expect("Second note not found");

    let qipu = crate::eval_helpers::get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");
    let output = std::process::Command::new(qipu_abs)
        .args([
            "link",
            "add",
            first_note_id,
            second_note_id,
            "--type",
            "related",
        ])
        .current_dir(&env_root)
        .output()
        .expect("failed to run qipu link add");
    assert!(
        output.status.success(),
        "Link add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let link_scenario_pass = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::LinkExists {
                from: first_note_id.to_string(),
                to: second_note_id.to_string(),
                link_type: "related".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&link_scenario_pass, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let tag_scenario_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::TagExists {
                tag: "nonexistent".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&tag_scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);

    let qipu = crate::eval_helpers::get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");
    let output = std::process::Command::new(&qipu_abs)
        .args(["create", "Important note", "--tag", "important"])
        .current_dir(&env_root)
        .output()
        .expect("failed to run qipu create");
    assert!(
        output.status.success(),
        "Create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tag_scenario_pass = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::TagExists {
                tag: "important".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&tag_scenario_pass, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let content_scenario_pass = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::ContentContains {
                id: first_note_id.to_string(),
                substring: "test note".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&content_scenario_pass, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 1);

    let content_scenario_fail = Scenario {
        name: "test".to_string(),
        description: "test".to_string(),
        template_folder: "test".to_string(),
        task: Task {
            prompt: "test".to_string(),
        },
        evaluation: Evaluation {
            gates: vec![Gate::ContentContains {
                id: first_note_id.to_string(),
                substring: "nonexistent".to_string(),
            }],
            judge: None,
        },
        tier: 0,
        tool_matrix: None,
        setup: None,
        tags: vec![],
        run: None,
    };
    let metrics = evaluate(&content_scenario_fail, &env_root, false).unwrap();
    assert_eq!(metrics.gates_passed, 0);
}
