use crate::support::qipu;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_compact_apply_mixed_sources() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    for i in 1..=5 {
        let note_content = format!(
            r#"---
id: qp-note{}
title: Source Note {}
type: permanent
---
This is source note {} content."#,
            i, i, i
        );
        fs::write(
            dir.path()
                .join(format!(".qipu/notes/qp-note{}-source-note-{}.md", i, i)),
            note_content,
        )
        .unwrap();
    }

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes notes 1-5."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let notes_file = dir.path().join("notes.txt");
    let mut file = fs::File::create(&notes_file).unwrap();
    writeln!(file, "qp-note3").unwrap();
    writeln!(file, "qp-note4").unwrap();

    let stdin_input = "qp-note1\nqp-note5\nqp-note5\n";
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--note",
            "qp-note1",
            "--note",
            "qp-note2",
            "--notes-file",
            notes_file.to_str().unwrap(),
            "--from-stdin",
        ])
        .write_stdin(stdin_input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacts 5 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));
    assert!(stdout.contains("qp-note4"));
    assert!(stdout.contains("qp-note5"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["count"], 5);
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 5);
}
