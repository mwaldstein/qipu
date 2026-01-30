use crate::support::{extract_id, qipu, setup_test_dir};

#[test]
fn test_link_tree_max_fanout_truncation() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D", "E"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-fanout",
            "2",
            "--direction",
            "out",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_fanout");

    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);

    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}

#[test]
fn test_link_tree_max_fanout_records_truncation() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D", "E"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "tree",
            &root,
            "--max-fanout",
            "2",
            "--direction",
            "out",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=true"),
        "Expected truncated=true in header"
    );

    assert!(stdout.contains(&format!("N {}", root)));

    let edge_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("E "))
        .collect();

    assert_eq!(edge_lines.len(), 2, "Expected exactly 2 edges");
}

#[test]
fn test_link_tree_max_fanout_direction_in() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let target = extract_id(&output);

    let mut source_ids = Vec::new();
    for label in ["A", "B", "C", "D", "E"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let source_id = extract_id(&output);
        source_ids.push(source_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &source_id, &target, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &target,
            "--max-fanout",
            "2",
            "--direction",
            "in",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_fanout");

    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);

    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}

#[test]
fn test_link_tree_max_fanout_direction_both() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D", "E"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-fanout",
            "2",
            "--direction",
            "both",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_fanout");

    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);

    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 4);
}
