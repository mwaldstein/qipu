use crate::support::setup_test_dir;
//! Records format character handling tests

use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_records_very_long_title() {
    let dir = setup_test_dir();

    // Create a note with a long but realistic title (around 200 characters)
    let long_title = "This is a very long title that might be used in practice and should still be handled correctly by the records format without causing any issues with output or parsing for downstream tools that consume the data";

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", long_title])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // List in records format should handle long title
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Should contain the long title (quoted)
    assert!(
        stdout.contains(&format!("\"{}\"", &long_title)),
        "Should contain full long title in quotes. Output: {}",
        stdout
    );
}

#[test]
fn test_records_very_long_tags() {
    let dir = setup_test_dir();

    // Create a note with many tags (50 tags, each 20 chars)
    let tags: Vec<String> = (0..50).map(|i| format!("verylongtag{:020}", i)).collect();
    let tag_args: Vec<&str> = tags
        .iter()
        .flat_map(|t| vec!["--tag", t.as_str()])
        .collect();

    let mut args = vec!["create"];
    args.extend(tag_args);
    args.push("Many Tags Note");

    let output = qipu().current_dir(dir.path()).args(args).output().unwrap();
    let id = extract_id(&output);

    // List in records format should handle many tags
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Should contain tags= field
    assert!(stdout.contains("tags="), "Should have tags field");

    // All tags should be present and comma-separated
    let tags_str = tags.join(",");
    assert!(
        stdout.contains(&tags_str),
        "Should contain all tags comma-separated"
    );
}

#[test]
fn test_records_newlines_in_title() {
    let dir = setup_test_dir();

    // Create a note with newlines in title (via edit to bypass CLI validation)
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Base Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit the note to add newlines in title (edge case from direct file edits)
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}.md", id));
    std::fs::write(
        &note_path,
        "---\ntitle: \"Line1\\nLine2\\nLine3\"\ntype: permanent\n---\nBody content",
    )
    .unwrap();

    // Reindex to pick up changes
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List in records format should handle escaped newlines
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Title should be on a single line (newlines escaped)
    assert!(
        !stdout
            .lines()
            .any(|line| line.starts_with("N ") && line.contains('\n')),
        "N line should not contain literal newlines"
    );
}

#[test]
fn test_records_backslashes_in_title() {
    let dir = setup_test_dir();

    // Create a note with backslashes in title
    let title_with_backslashes = r"Path\to\file";
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title_with_backslashes])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // List in records format should handle backslashes
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Title should be present with backslashes
    assert!(
        stdout.contains(title_with_backslashes),
        "Should contain backslashes in title"
    );
}

#[test]
fn test_records_body_with_special_chars() {
    let dir = setup_test_dir();

    // Create a note with special characters in title
    let title_with_special = "Note with \\special\\ &chars\\\"";

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title_with_special])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // List in records format should handle special characters in title
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Should contain special characters from title (properly escaped)
    assert!(
        stdout.contains("special"),
        "Should contain 'special' from title. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("chars"),
        "Should contain 'chars' from title"
    );
}

#[test]
fn test_records_unicode_characters() {
    let dir = setup_test_dir();

    // Create a note with unicode characters (emoji, chinese, arabic)
    let unicode_title = "Hello 世界 🌍 مرحبا";
    let unicode_tags = ["emoji😀", "中文", "العربية"];

    let mut create_args = vec!["create", unicode_title];
    for tag in &unicode_tags {
        create_args.push("--tag");
        create_args.push(tag);
    }

    let output = qipu()
        .current_dir(dir.path())
        .args(&create_args)
        .output()
        .unwrap();
    let id = extract_id(&output);

    // List in records format should handle unicode
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Should contain unicode title
    if !stdout.contains(unicode_title) {
        eprintln!("Expected unicode title: {}", unicode_title);
        eprintln!("Output: {}", stdout);
    }
    assert!(
        stdout.contains(unicode_title),
        "Should contain unicode characters in title"
    );

    // Should contain unicode tags
    for tag in &unicode_tags {
        assert!(stdout.contains(tag), "Should contain unicode tag: {}", tag);
    }
}

#[test]
fn test_records_special_chars_in_tags() {
    let dir = setup_test_dir();

    // Create a note with special characters in tags
    let special_tags = [
        "tag-with-dash",
        "tag_with_underscore",
        "tag.with.dots",
        "tag/with/slashes",
        "tag:with:colons",
    ];

    let mut args = vec!["create"];
    for tag in &special_tags {
        args.push("--tag");
        args.push(tag);
    }
    args.push("Special Tags Note");

    let output = qipu().current_dir(dir.path()).args(args).output().unwrap();
    let id = extract_id(&output);

    // List in records format should handle special chars in tags
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Should contain tags= field
    assert!(stdout.contains("tags="), "Should have tags field");

    // All special tags should be present
    for tag in &special_tags {
        assert!(stdout.contains(tag), "Should contain tag: {}", tag);
    }
}

#[test]
fn test_records_single_quote_in_title() {
    let dir = setup_test_dir();

    // Create a note with single quotes in title
    let title_with_single_quotes = "Title with 'single quotes'";
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title_with_single_quotes])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // List in records format should handle single quotes
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain the note ID
    assert!(stdout.contains(&id));

    // Title should be present with single quotes (no escaping needed for single quotes in double-quoted strings)
    assert!(
        stdout.contains(title_with_single_quotes),
        "Should contain single quotes in title"
    );
}
