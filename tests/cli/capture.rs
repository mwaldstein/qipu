use crate::cli::support::{extract_id_from_bytes, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Capture command tests
// ============================================================================

#[test]
fn test_capture_basic() {
    let dir = tempdir().unwrap();

    // Init first
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture note from stdin
    qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("This is my captured note\nWith multiple lines")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_capture_with_title() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with explicit title
    qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "My Custom Title"])
        .write_stdin("Content goes here")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_capture_with_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with specific type
    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "literature"])
        .write_stdin("Literature note content")
        .assert()
        .success();
}

#[test]
fn test_capture_with_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with tags
    qipu()
        .current_dir(dir.path())
        .args(["capture", "--tag", "docs", "--tag", "test"])
        .write_stdin("Tagged capture content")
        .assert()
        .success();
}

#[test]
fn test_capture_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with JSON output
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture", "--title", "JSON Capture"])
        .write_stdin("JSON test content")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\": \"JSON Capture\""))
        .stdout(predicate::str::contains("\"type\":"));
}

#[test]
fn test_capture_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with records output
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "capture", "--title", "Records Test"])
        .write_stdin("Records content")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("H qipu=1"))
        .stdout(predicate::str::contains("N qp-"));
}

#[test]
fn test_capture_default_type_fleeting() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture without specifying type (should default to fleeting)
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("Default type test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"type\": \"fleeting\""));
}

#[test]
fn test_capture_content_preservation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture and verify content is preserved
    let content = "Line 1\nLine 2\n\nLine 4 with spacing";
    let output = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Content Test"])
        .write_stdin(content)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id = extract_id_from_bytes(&output);

    // Read the note back and verify content
    let show_output = qipu()
        .current_dir(dir.path())
        .args(["show", &note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let show_str = String::from_utf8(show_output).unwrap();
    assert!(show_str.contains("Line 1"));
    assert!(show_str.contains("Line 2"));
    assert!(show_str.contains("Line 4 with spacing"));
}

#[test]
fn test_capture_empty_content() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with empty stdin should still create note
    qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Empty Note"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_capture_auto_title_from_content() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture without title - should generate from content
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("Auto generated title from this line\nMore content")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Auto generated title from this line"));
}

#[test]
fn test_capture_auto_title_from_heading() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with markdown heading - should extract heading as title
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("# My Heading\n\nSome content below")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("My Heading"));
}

#[test]
fn test_capture_auto_title_empty_content() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with no title and empty content - should use fallback
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Untitled capture"));
}

#[test]
fn test_capture_with_provenance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with provenance fields
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Provenance Test",
            "--source",
            "https://example.com",
            "--author",
            "Test Author",
            "--generated-by",
            "test-agent",
        ])
        .write_stdin("Content with provenance")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id = extract_id_from_bytes(&output);

    // Verify provenance fields are saved
    let note_path = dir.path().join(".qipu").join("notes");
    let note_file = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content = fs::read_to_string(note_file).unwrap();
    assert!(note_content.contains("source: https://example.com"));
    assert!(note_content.contains("author: Test Author"));
    assert!(note_content.contains("generated_by: test-agent"));
    // Per spec: LLM-generated notes should default to verified: false
    assert!(note_content.contains("verified: false"));
}

#[test]
fn test_capture_web_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Test 1: When source is provided but author is not, should default to "Qipu Clipper"
    let output1 = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Web Capture Test 1",
            "--source",
            "https://example.com/article",
        ])
        .write_stdin("Content from web")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id1 = extract_id_from_bytes(&output1);
    let note_path = dir.path().join(".qipu").join("notes");
    let note_file1 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id1)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content1 = fs::read_to_string(note_file1).unwrap();
    assert!(note_content1.contains("source: https://example.com/article"));
    assert!(note_content1.contains("author: Qipu Clipper"));

    // Test 2: When source is provided and author is explicitly set, should use provided author
    let output2 = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Web Capture Test 2",
            "--source",
            "https://example.com/article2",
            "--author",
            "John Doe",
        ])
        .write_stdin("Content from web with author")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id2 = extract_id_from_bytes(&output2);
    let note_file2 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id2)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content2 = fs::read_to_string(note_file2).unwrap();
    assert!(note_content2.contains("source: https://example.com/article2"));
    assert!(note_content2.contains("author: John Doe"));
    assert!(!note_content2.contains("author: Qipu Clipper"));

    // Test 3: When neither source nor author is provided, should not set author
    let output3 = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Non-web Capture Test"])
        .write_stdin("Content without source")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id3 = extract_id_from_bytes(&output3);
    let note_file3 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id3)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content3 = fs::read_to_string(note_file3).unwrap();
    assert!(!note_content3.contains("source:"));
    assert!(!note_content3.contains("author:"));
}

#[test]
fn test_capture_with_id() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with explicit ID
    qipu()
        .current_dir(dir.path())
        .args(["capture", "--id", "qp-custom123", "--title", "Custom ID"])
        .write_stdin("Content with custom ID")
        .assert()
        .success()
        .stdout(predicate::str::contains("qp-custom123"));
}

#[test]
fn test_capture_verbose_output() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with verbose flag
    qipu()
        .current_dir(dir.path())
        .args(["--verbose", "capture", "--title", "Verbose Test"])
        .write_stdin("Verbose output test")
        .assert()
        .success()
        .stdout(predicate::str::contains("qp-"))
        .stdout(predicate::str::contains("Captured:"));
}

#[test]
fn test_capture_multiple_tags_json() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with multiple tags and verify in JSON
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "capture", "--tag", "alpha", "--tag", "beta", "--tag", "gamma",
        ])
        .write_stdin("Multi-tag test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"tags\":"));
    assert!(output_str.contains("alpha"));
    assert!(output_str.contains("beta"));
    assert!(output_str.contains("gamma"));
}

#[test]
fn test_capture_records_with_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with tags in records format
    qipu()
        .current_dir(dir.path())
        .args([
            "--format", "records", "capture", "--tag", "foo", "--tag", "bar",
        ])
        .write_stdin("Records tags test")
        .assert()
        .success()
        .stdout(predicate::str::contains("tags=foo,bar"));
}

#[test]
fn test_capture_records_no_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture without tags in records format should show "-"
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "capture"])
        .write_stdin("No tags test")
        .assert()
        .success()
        .stdout(predicate::str::contains("tags=-"));
}

#[test]
fn test_capture_json_with_provenance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Capture with provenance fields and JSON output
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "capture",
            "--title",
            "Provenance JSON Test",
            "--source",
            "https://example.com/article",
            "--author",
            "Jane Doe",
            "--generated-by",
            "gpt-4",
            "--prompt-hash",
            "abc123",
            "--verified",
            "true",
        ])
        .write_stdin("Content with full provenance")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Verify all provenance fields are present in JSON output
    assert!(output_str.contains("\"source\": \"https://example.com/article\""));
    assert!(output_str.contains("\"author\": \"Jane Doe\""));
    assert!(output_str.contains("\"generated_by\": \"gpt-4\""));
    assert!(output_str.contains("\"prompt_hash\": \"abc123\""));
    assert!(output_str.contains("\"verified\": true"));
}
