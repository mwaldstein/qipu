use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_tree_max_hops() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let mut ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Node {}", i)])
            .output()
            .unwrap();
        ids.push(extract_id(&output));
    }

    for i in 0..4 {
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &ids[i], &ids[i + 1], "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &ids[0], "--max-hops", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node 0"))
        .stdout(predicate::str::contains("Node 1"))
        .stdout(predicate::str::contains("Node 2"))
        .stdout(predicate::str::contains("Node 3").not());
}

#[test]
fn test_link_tree_max_hops_reports_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let mut ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Node {}", i)])
            .output()
            .unwrap();
        ids.push(extract_id(&output));
    }

    for i in 0..4 {
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &ids[i], &ids[i + 1], "--type", "related"])
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
            &ids[0],
            "--max-hops",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_hops");

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert!(note_ids.contains(&ids[0]));
    assert!(note_ids.contains(&ids[1]));
    assert!(note_ids.contains(&ids[2]));
    assert!(!note_ids.contains(&ids[3]));
    assert!(!note_ids.contains(&ids[4]));
}

#[test]
fn test_link_tree_max_nodes_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C"] {
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

        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("{} Child", label)])
            .output()
            .unwrap();
        let grandchild_id = extract_id(&output);

        qipu()
            .current_dir(dir.path())
            .args([
                "link",
                "add",
                &child_id,
                &grandchild_id,
                "--type",
                "related",
            ])
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
            "--max-nodes",
            "3",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_nodes");

    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);
}

#[test]
fn test_link_tree_max_edges_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D"] {
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
            "--max-edges",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_edges");

    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}

#[test]
fn test_link_tree_max_fanout_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
fn test_link_tree_records_max_chars_no_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child = qipu()
        .current_dir(dir.path())
        .args(["create", "Child"])
        .output()
        .unwrap();
    let id_child = extract_id(&output_child);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child, "--type", "related"])
        .assert()
        .success();

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
            &id_root,
            "--max-chars",
            "10000",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=false"),
        "Expected truncated=false in header"
    );

    assert!(stdout.contains(&format!("N {}", id_root)));
    assert!(stdout.contains(&format!("N {}", id_child)));

    assert!(stdout.contains(&format!("E {} related {}", id_root, id_child)));
}

#[test]
fn test_link_tree_records_max_chars_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let mut child_ids = Vec::new();
    for i in 1..=5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Child {}", i)])
            .output()
            .unwrap();
        let id = extract_id(&output);
        child_ids.push(id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &id_root, &id, "--type", "related"])
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
            &id_root,
            "--max-chars",
            "200",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=true"),
        "Expected truncated=true in header"
    );

    assert!(stdout.contains("H qipu=1 records=1"));

    let total_chars = stdout.len();
    assert!(
        total_chars <= 200,
        "Output exceeded max-chars budget: {} > 200",
        total_chars
    );
}

#[test]
fn test_link_tree_records_max_chars_header_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

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
            &id_root,
            "--max-chars",
            "120",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=true"),
        "Expected truncated=true in header"
    );

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Expected only header line, got {} lines",
        lines.len()
    );
    assert!(lines[0].starts_with("H "));

    assert!(stdout.len() <= 120);
}
