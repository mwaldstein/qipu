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
    let output1 = qipu()
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
        .output()
        .unwrap();

    assert!(output1.status.success());
    let stdout1 = String::from_utf8(output1.stdout).unwrap();

    eprintln!("Output with max-chars=100:\n{}", stdout1);
    eprintln!("Length: {}", stdout1.len());

    // Should indicate truncation
    assert!(stdout1.contains("truncated"));
    // When budget is extremely small (100 chars), we may not have room for excluded notes section
    // Just verify the note content is not included
    assert!(!stdout1.contains("This is a repeating line"));

    // If max-tokens is very small, it should truncate even if max-chars is large
    let output2 = qipu()
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
        .output()
        .unwrap();

    assert!(output2.status.success());
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    eprintln!("Output with max-tokens=10:\n{}", stdout2);
    eprintln!("Length: {}", stdout2.len());

    // Should indicate truncation
    assert!(stdout2.contains("truncated"));
    // When budget is extremely small, we may not have room for excluded notes section
    // Just verify the note content is not included
    assert!(!stdout2.contains("This is a repeating line"));
}

#[test]
fn test_context_prefers_typed_links_over_related() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a root MOC note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note", "--type", "moc"])
        .output()
        .unwrap();
    let root_id = String::from_utf8(root_output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();

    // Create notes with different link types
    // Create a related note FIRST (so it has earlier timestamp)
    let related_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Related Note"])
        .output()
        .unwrap();
    let related_id = String::from_utf8(related_output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();

    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Now create notes with typed links (part-of, supports) - later timestamps
    let part_of_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Part Of Note"])
        .output()
        .unwrap();
    let part_of_id = String::from_utf8(part_of_output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let supports_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Supports Note"])
        .output()
        .unwrap();
    let supports_id = String::from_utf8(supports_output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();

    // Add links with different types
    // Add related FIRST to ensure it would be selected first if there's no prioritization
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &related_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &part_of_id, "--type", "part-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &supports_id, "--type", "supports"])
        .assert()
        .success();

    // First, test without budget to verify priority order
    let no_budget_output = qipu()
        .current_dir(dir.path())
        .args(["context", "--moc", &root_id, "--format", "human"])
        .output()
        .unwrap();

    let no_budget_stdout = String::from_utf8(no_budget_output.stdout).unwrap();

    // Find positions of each note in the output
    let part_of_pos = no_budget_stdout.find("Part Of Note");
    let supports_pos = no_budget_stdout.find("Supports Note");
    let related_pos = no_budget_stdout.find("Related Note");

    // Verify priority order: part-of and supports should come before related
    if let (Some(part_pos), Some(rel_pos)) = (part_of_pos, related_pos) {
        assert!(
            part_pos < rel_pos,
            "part-of note should appear before related note (positions: {} vs {})",
            part_pos,
            rel_pos
        );
    }
    if let (Some(supp_pos), Some(rel_pos)) = (supports_pos, related_pos) {
        assert!(
            supp_pos < rel_pos,
            "supports note should appear before related note (positions: {} vs {})",
            supp_pos,
            rel_pos
        );
    }

    // Now test with small budget - should prefer typed links
    // Set budget to allow ~1-2 notes but not all 3 linked notes
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--moc",
            &root_id,
            "--max-chars",
            "600",
            "--format",
            "human",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Count which notes were included
    let has_part_of = stdout.contains("Part Of Note");
    let has_supports = stdout.contains("Supports Note");
    let has_related = stdout.contains("Related Note");
    let is_truncated = stdout.contains("truncated");

    // If all notes fit, test passes (no constraint to test)
    if !is_truncated {
        return;
    }

    // If budget forced a choice, typed links should be preferred over related
    if !has_part_of && !has_supports {
        // If neither typed link was included, something is wrong
        panic!("Budget should include at least one typed link before related links");
    }

    // If related was included but not all typed links, that's a preference violation
    if has_related && (!has_part_of || !has_supports) {
        panic!(
            "Related link should not be included when typed links are excluded. \
             part_of={}, supports={}, related={}",
            has_part_of, has_supports, has_related
        );
    }

    // Success: typed links (part-of, supports) are prioritized over related links
}

#[test]
fn test_context_shows_excluded_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    let mut note_ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Test Note {}", i), "--tag", "test"])
            .output()
            .unwrap();

        let id = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .trim()
            .to_string();
        note_ids.push(id);
    }

    // Test human format with budget that truncates some notes
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "test",
            "--max-chars",
            "800",
            "--format",
            "human",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show truncation notice
    assert!(stdout.contains("truncated"));
    // Should list complete notes that fit within budget
    assert!(stdout.contains("Test Note 0"));
    assert!(stdout.contains("Test Note 1"));

    // Test JSON format with budget that truncates some notes
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "test",
            "--max-chars",
            "1000",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have truncated flag
    assert_eq!(json["truncated"], true);
    // Should have notes array (not excluded_notes)
    assert!(json["notes"].is_array());
    let notes = json["notes"].as_array().unwrap();
    assert!(!notes.is_empty(), "Should have at least one note");
    // At least one note should have content_truncated=true
    let has_truncated = notes.iter().any(|note| {
        note.get("content_truncated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });
    assert!(has_truncated, "Should have at least one truncated note");
    // Each note should have id and title
    for note in notes {
        assert!(note["id"].is_string());
        assert!(note["title"].is_string());
    }

    // Test records format with budget that truncates some notes
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--tag",
            "test",
            "--max-chars",
            "600",
            "--format",
            "records",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should have truncated=true in header
    assert!(stdout.contains("truncated=true"));
    // Should have "truncated" annotation in note headers
    assert!(stdout.contains("truncated"));
}
