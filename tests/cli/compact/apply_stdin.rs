use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_compact_apply_from_stdin() {
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

    let stdin_input = "qp-note1\nqp-note2\nqp-note3\n";
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", "qp-digest", "--from-stdin"])
        .write_stdin(stdin_input)
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

    let stdin_input2 = "qp-note1\nqp-note2\n";
    let output = qipu()
        .current_dir(dir2.path())
        .args([
            "--format",
            "json",
            "compact",
            "apply",
            "qp-digest2",
            "--from-stdin",
        ])
        .write_stdin(stdin_input2)
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

    let stdin_input3 = "qp-note1\n";
    let output = qipu()
        .current_dir(dir3.path())
        .args([
            "--format",
            "records",
            "compact",
            "apply",
            "qp-digest3",
            "--from-stdin",
        ])
        .write_stdin(stdin_input3)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.apply digest=qp-digest3 count=1"));
    assert!(stdout.contains("D compacted qp-note1"));
}
