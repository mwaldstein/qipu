use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Budget and truncation tests
// ============================================================================

#[test]
fn test_context_max_chars() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    for i in 0..5 {
        qipu()
            .current_dir(dir.path())
            .args(["create", "--tag", "budget", &format!("Budget Note {}", i)])
            .assert()
            .success();
    }

    // Get context with small budget - should truncate
    qipu()
        .current_dir(dir.path())
        .args(["context", "--tag", "budget", "--max-chars", "1200"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Budget Note")) // At least one note
        .stdout(predicate::str::contains("truncated")); // Should indicate truncation
}

#[test]
fn test_context_budget_exact() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes with known content
    for i in 0..10 {
        qipu()
            .current_dir(dir.path())
            .args(["create", "--tag", "budget-test", &format!("Note {}", i)])
            .assert()
            .success();
    }

    // Test budget enforcement in human format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "budget-test",
            "--max-chars",
            "800",
            "--format",
            "human",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify output doesn't exceed budget
    assert!(
        stdout.len() <= 800,
        "Output size {} exceeds budget 800",
        stdout.len()
    );

    // Should indicate truncation since we have many notes
    assert!(
        stdout.contains("truncated"),
        "Output should indicate truncation"
    );

    // Test budget enforcement in JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "budget-test",
            "--max-chars",
            "1000",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify output doesn't exceed budget
    assert!(
        stdout.len() <= 1000,
        "JSON output size {} exceeds budget 1000",
        stdout.len()
    );

    // Parse JSON and check truncated flag
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["truncated"], true, "Truncated flag should be true");

    // Test budget enforcement in records format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "budget-test",
            "--max-chars",
            "600",
            "--format",
            "records",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify output doesn't exceed budget
    assert!(
        stdout.len() <= 600,
        "Records output size {} exceeds budget 600",
        stdout.len()
    );

    // Should indicate truncation in header
    assert!(
        stdout.contains("truncated=true"),
        "Records output should indicate truncation in header"
    );
}

#[test]
fn test_context_max_tokens() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    for i in 0..5 {
        qipu()
            .current_dir(dir.path())
            .args([
                "create",
                "--tag",
                "token-budget",
                &format!("Token Note {}", i),
            ])
            .assert()
            .success();
    }

    // Get context with small token budget - should truncate
    // A typical small note is ~50-100 tokens with headers.
    // 150 tokens should allow about 1-2 notes.
    qipu()
        .current_dir(dir.path())
        .args(["context", "--tag", "token-budget", "--max-tokens", "150"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Token Note")) // At least one note
        .stdout(predicate::str::contains("truncated")); // Should indicate truncation
}

#[test]
fn test_context_max_tokens_and_chars() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a large note
    let mut large_body = String::new();
    for _ in 0..100 {
        large_body.push_str("This is a repeating line to increase size. ");
    }

    qipu()
        .current_dir(dir.path())
        .args(["create", "Large Note", "--tag", "both-budget"])
        .write_stdin(large_body)
        .assert()
        .success();

    // If max-chars is very small, it should truncate even if max-tokens is large
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "both-budget",
            "--max-chars",
            "100",
            "--max-tokens",
            "10000",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("truncated"))
        .stdout(predicate::str::contains("Large Note").not());

    // If max-tokens is very small, it should truncate even if max-chars is large
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "both-budget",
            "--max-chars",
            "10000",
            "--max-tokens",
            "10",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("truncated"))
        .stdout(predicate::str::contains("Large Note").not());
}
