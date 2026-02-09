use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_pack_format_s_prefix_means_sources() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("create")
        .arg("Note with Sources")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    for entry in walkdir::WalkDir::new(dir.path()) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                let updated_content =
                    content.replace("## Notes\n", "## Notes\n\nsummary: \"This is a summary\"\n");
                fs::write(entry.path(), updated_content).unwrap();
                break;
            }
        }
    }

    let note_path = dir.path().join(".qipu/notes/qp-citation-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-citation\ntitle: Citation Note\nsources:\n  - url: https://example.com/paper\n    title: Research Paper\n    accessed: 2024-01-15\n---\nCitation body content",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .arg("--rebuild")
        .assert()
        .success();

    let pack_file = dir.path().join("test.pack");
    qipu()
        .current_dir(dir.path())
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .assert()
        .success();

    let pack_content = fs::read_to_string(&pack_file).unwrap();

    assert!(
        pack_content.contains("S qp-citation url=https://example.com/paper"),
        "Pack format should use S prefix for Sources"
    );
    assert!(
        pack_content.contains("title=\"Research Paper\""),
        "Pack format should include source title"
    );
    assert!(
        pack_content.contains("accessed=2024-01-15"),
        "Pack format should include source accessed date"
    );

    assert!(
        !pack_content.contains(&format!("S {} This is a summary", note_id)),
        "Pack format should NOT use S prefix for Summary"
    );
}

#[test]
fn test_context_format_s_prefix_means_summary() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-source-note.md");
    let note_content = "---\nid: qp-aaaa\ntitle: Note with Summary\nsources:\n  - url: https://example.com/paper\n    title: Research Paper\n    accessed: 2024-01-15\n---\nNote body content";
    fs::write(&note_path, note_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .arg("--rebuild")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("context")
        .arg("--note")
        .arg("qp-aaaa")
        .arg("--format")
        .arg("records")
        .arg("--with-body")
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        output_str.contains("S qp-aaaa Note body content"),
        "Context format should use S prefix for summary/first paragraph"
    );
    assert!(
        output_str.contains("Note with Summary"),
        "Should include note title"
    );
    assert!(
        !output_str.contains("S qp-aaaa url="),
        "Context format should NOT use S prefix for Sources"
    );
    assert!(
        output_str.contains("D source url=https://example.com/paper"),
        "Context format should use D prefix for Sources"
    );
}

#[test]
fn test_truncation_header_distinct_from_s_lines() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes to trigger truncation
    for i in 0..5 {
        let note_path = dir.path().join(format!(".qipu/notes/test-note-{}.md", i));
        let content = format!(
            "---\nid: qp-test{}\ntitle: Test Note {}\ntags: [s-prefix-test]\nsources:\n  - url: https://example.com/paper{}\n    title: Paper {}\n---\nBody content for note {}",
            i, i, i, i, i
        );
        fs::write(&note_path, content).unwrap();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .arg("--rebuild")
        .assert()
        .success();

    // Get context with budget that triggers truncation
    let output = qipu()
        .current_dir(dir.path())
        .arg("context")
        .arg("--tag")
        .arg("s-prefix-test")
        .arg("--format")
        .arg("records")
        .arg("--with-body")
        .arg("--max-chars")
        .arg("800")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Verify truncation is indicated in header, not via S-lines
    assert!(
        output_str.contains("truncated=true"),
        "Header should indicate truncation with truncated=true"
    );

    // Verify S-lines contain actual summary content, not truncation markers
    let s_line_count = output_str.lines().filter(|l| l.starts_with("S ")).count();
    assert!(
        s_line_count > 0,
        "Should have S-lines with summary content when notes have body"
    );

    // Verify S-lines don't contain URL patterns (which would be pack format semantics)
    let s_lines_with_url: Vec<&str> = output_str
        .lines()
        .filter(|l| l.starts_with("S "))
        .filter(|l| l.contains("url="))
        .collect();
    assert!(
        s_lines_with_url.is_empty(),
        "Context S-lines should NOT contain url= (pack format semantics). Found: {:?}",
        s_lines_with_url
    );
}

#[test]
fn test_s_prefix_semantics_with_different_content_types() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Note with sources only (no summary in frontmatter)
    let note_with_sources = dir.path().join(".qipu/notes/with-sources.md");
    fs::write(
        &note_with_sources,
        "---\nid: qp-sources\ntitle: Note With Sources\ntags: [content-test]\nsources:\n  - url: https://example.com/source1\n    title: Source One\n  - url: https://example.com/source2\n    title: Source Two\n---\nThis is the body content.",
    )
    .unwrap();

    // Note without sources but with summary in frontmatter
    let note_with_summary = dir.path().join(".qipu/notes/with-summary.md");
    fs::write(
        &note_with_summary,
        "---\nid: qp-summary\ntitle: Note With Summary\ntags: [content-test]\nsummary: \"Frontmatter summary\"\n---\nThis is different body content.",
    )
    .unwrap();

    // Note with neither sources nor summary
    let plain_note = dir.path().join(".qipu/notes/plain.md");
    fs::write(
        &plain_note,
        "---\nid: qp-plain\ntitle: Plain Note\ntags: [content-test]\n---\nPlain note body here.",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .arg("--rebuild")
        .assert()
        .success();

    // Test pack format
    let pack_file = dir.path().join("test.pack");
    qipu()
        .current_dir(dir.path())
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .assert()
        .success();

    let pack_content = fs::read_to_string(&pack_file).unwrap();

    // Pack format: S = Sources
    assert!(
        pack_content.contains("S qp-sources url=https://example.com/source1"),
        "Pack format should use S for sources (note with sources)"
    );
    assert!(
        pack_content.contains("S qp-sources url=https://example.com/source2"),
        "Pack format should use S for all sources"
    );
    // Plain note should not have S-lines in pack format (no sources)
    let plain_s_lines: Vec<&str> = pack_content
        .lines()
        .filter(|l| l.starts_with("S qp-plain "))
        .collect();
    assert!(
        plain_s_lines.is_empty(),
        "Pack format should not have S-lines for notes without sources"
    );

    // Test context format
    let output = qipu()
        .current_dir(dir.path())
        .arg("context")
        .arg("--tag")
        .arg("content-test")
        .arg("--format")
        .arg("records")
        .arg("--with-body")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Context command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Context format: S = Summary (body content)
    assert!(
        output_str.contains("S qp-plain Plain note body here."),
        "Context format should use S for summary/body (plain note)"
    );
    assert!(
        output_str.contains("S qp-sources This is the body content."),
        "Context format should use S for summary even when note has sources"
    );
    // Sources should use D-lines in context format
    assert!(
        output_str.contains("D source url=https://example.com/source1"),
        "Context format should use D prefix for sources"
    );
}

#[test]
fn test_s_prefix_not_confused_with_truncation_in_context() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with "truncated" in its body to test no confusion
    let note_path = dir.path().join(".qipu/notes/truncated-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-trunc\ntitle: Note About Truncation\n---\nThis document discusses how truncation works in data processing and why truncated=false is different from truncated=true in various scenarios.",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .arg("--rebuild")
        .assert()
        .success();

    // Test without budget (truncated=false)
    let output_full = qipu()
        .current_dir(dir.path())
        .arg("context")
        .arg("--note")
        .arg("qp-trunc")
        .arg("--format")
        .arg("records")
        .arg("--with-body")
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output_full.stdout);

    // Should have truncated=false in header
    assert!(
        output_str.contains("truncated=false"),
        "Header should show truncated=false when no budget constraint"
    );

    // Should still have S-line with the word "truncated" in content
    let s_lines: Vec<&str> = output_str.lines().filter(|l| l.starts_with("S ")).collect();
    assert!(
        !s_lines.is_empty(),
        "Should have S-line with summary content"
    );
    assert!(
        s_lines[0].contains("truncated"),
        "S-line should contain the word 'truncated' from note body"
    );

    // Verify the S-line is properly formed (S <id> <content>)
    assert!(
        s_lines[0].starts_with("S qp-trunc "),
        "S-line should start with 'S qp-trunc ' followed by content"
    );
}
