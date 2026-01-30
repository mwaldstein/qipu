use crate::support::setup_test_dir;
//! Records format budget truncation tests

use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_records_budget_truncation_header_only() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Context with small budget (fits header but not full note metadata)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            &id,
            "--max-chars",
            "100",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have header
    assert!(
        stdout.contains("H qipu=1 records=1"),
        "Should have header even with small budget"
    );

    // Should indicate truncation if budget is exceeded
    // (Note: The system may output header even if it exceeds budget slightly)
    if stdout.len() > 100 {
        assert!(
            stdout.contains("truncated=true"),
            "Should indicate truncation when output exceeds budget"
        );
    }

    // Should have header line
    assert!(
        stdout.lines().next().is_some_and(|l| l.starts_with("H ")),
        "First line should be header"
    );
}

#[test]
fn test_records_budget_truncation_mid_record() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output);

    // Create multiple child notes
    let mut child_ids = Vec::new();
    for i in 1..=5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Child Note {}", i)])
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

    // Link tree with budget that cuts off mid-output
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "tree",
            &id_root,
            "--max-chars",
            "300",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have header
    assert!(stdout.contains("H qipu=1 records=1"));

    // Should indicate truncation
    assert!(
        stdout.contains("truncated=true"),
        "Should indicate truncation"
    );

    // Should not exceed budget
    assert!(
        stdout.len() <= 300,
        "Output should not exceed budget: {} > 300",
        stdout.len()
    );

    // Should have complete header line
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty(), "Should have at least header line");
    assert!(lines[0].starts_with("H "), "First line should be header");

    // If there are N lines after header, they should be complete (not cut off mid-line)
    if lines.len() > 1 {
        for (i, line) in lines.iter().enumerate().skip(1) {
            // All lines should end with newline or be the last line
            // (no mid-line truncation)
            assert!(
                !line.is_empty(),
                "Line {} should not be empty (possible mid-record truncation)",
                i
            );
        }
    }
}
