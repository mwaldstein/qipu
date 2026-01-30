use crate::support::{extract_id, qipu, setup_test_dir};
use tempfile::tempdir;

#[test]
fn test_link_tree_records_max_chars_no_truncation() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
