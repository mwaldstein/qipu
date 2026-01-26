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
