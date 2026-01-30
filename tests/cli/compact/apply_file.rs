use crate::cli::support::qipu;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_compact_apply_notes_file() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note1_content = r#"---
id: qp-note1
title: Source Note 1
type: permanent
---
This is source note 1 content."#;

    let note2_content = r#"---
id: qp-note2
title: Source Note 2
type: permanent
---
This is source note 2 content."#;

    let note3_content = r#"---
id: qp-note3
title: Source Note 3
type: permanent
---
This is source note 3 content."#;

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes notes 1-3."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note3-source-note-3.md"),
        note3_content,
    )
    .unwrap();
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
    writeln!(file, "qp-note1").unwrap();
    writeln!(file, "qp-note2").unwrap();
    writeln!(file, "qp-note3").unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Applied compaction:"));
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Compacts 3 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Direct compaction count: 3"));

    let dir2 = tempdir().unwrap();
    qipu()
        .current_dir(dir2.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir2.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-digest2-digest-note-2.md"),
        r#"---
id: qp-digest2
title: Digest Note 2
type: permanent
---
Another digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir2.path())
        .arg("index")
        .assert()
        .success();

    let notes_file2 = dir2.path().join("notes2.txt");
    let mut file2 = fs::File::create(&notes_file2).unwrap();
    writeln!(file2, "qp-note1").unwrap();
    writeln!(file2, "qp-note2").unwrap();

    let output = qipu()
        .current_dir(dir2.path())
        .args([
            "--format",
            "json",
            "compact",
            "apply",
            "qp-digest2",
            "--notes-file",
            notes_file2.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest2");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-note1")));
    assert!(compacts.contains(&serde_json::json!("qp-note2")));

    let dir3 = tempdir().unwrap();
    qipu()
        .current_dir(dir3.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir3.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir3.path().join(".qipu/notes/qp-digest3-digest-note-3.md"),
        r#"---
id: qp-digest3
title: Digest Note 3
type: permanent
---
Third digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir3.path())
        .arg("index")
        .assert()
        .success();

    let notes_file3 = dir3.path().join("notes3.txt");
    let mut file3 = fs::File::create(&notes_file3).unwrap();
    writeln!(file3, "qp-note1").unwrap();

    let output = qipu()
        .current_dir(dir3.path())
        .args([
            "--format",
            "records",
            "compact",
            "apply",
            "qp-digest3",
            "--notes-file",
            notes_file3.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.apply digest=qp-digest3 count=1"));
    assert!(stdout.contains("D compacted qp-note1"));

    let dir4 = tempdir().unwrap();
    qipu()
        .current_dir(dir4.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir4.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir4.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir4.path().join(".qipu/notes/qp-digest4-digest-note-4.md"),
        r#"---
id: qp-digest4
title: Digest Note 4
type: permanent
---
Fourth digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir4.path())
        .arg("index")
        .assert()
        .success();

    let notes_file4 = dir4.path().join("notes4.txt");
    let mut file4 = fs::File::create(&notes_file4).unwrap();
    writeln!(file4, "qp-note1").unwrap();
    writeln!(file4).unwrap();
    writeln!(file4, "  ").unwrap();
    writeln!(file4, "qp-note2").unwrap();

    let output = qipu()
        .current_dir(dir4.path())
        .args([
            "compact",
            "apply",
            "qp-digest4",
            "--notes-file",
            notes_file4.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
}
