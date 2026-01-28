use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_compact_multi_level_chain() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note1_content = r#"---
id: qp-note1
title: Note 1
type: permanent
---
This is note 1 content."#;

    let note2_content = r#"---
id: qp-note2
title: Note 2
type: permanent
---
This is note 2 content."#;

    let note3_content = r#"---
id: qp-note3
title: Note 3
type: permanent
---
This is note 3 content."#;

    let digest1_content = r#"---
id: qp-digest1
title: Digest Level 1
type: permanent
---
Level 1 digest summarizing notes 1 and 2."#;

    let digest2_content = r#"---
id: qp-digest2
title: Digest Level 2
type: permanent
---
Level 2 digest summarizing digest1 and note3."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note3-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest1-digest-level-1.md"),
        digest1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest2-digest-level-2.md"),
        digest2_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest1",
            "--note",
            "qp-note1",
            "--note",
            "qp-note2",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest2",
            "--note",
            "qp-digest1",
            "--note",
            "qp-note3",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "status", "qp-note1"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacted by: Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Canonical: Digest Level 2 (qp-digest2)"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "status", "qp-digest1"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacted by: Digest Level 2 (qp-digest2)"));
    assert!(stdout.contains("Canonical: Digest Level 2 (qp-digest2)"));
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("Note 1 (qp-note1)"));
    assert!(stdout.contains("Note 2 (qp-note2)"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "status", "qp-digest2"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacted by: (none)"));
    assert!(stdout.contains("Canonical: (self)"));
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "status", "qp-note1"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["note_id"], "qp-note1");
    assert_eq!(json["compactor"], "qp-digest1");
    assert_eq!(json["canonical"], "qp-digest2");

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest1"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Digest: qp-digest1"));
    assert!(stdout.contains("Direct compaction count: 2"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest2"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Digest: qp-digest2"));
    assert!(stdout.contains("Direct compaction count: 2"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest2", "--compaction-depth", "3"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Nested compaction (depth 3):"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));
    assert!(stdout.contains("Note 1 (qp-note1)"));
    assert!(stdout.contains("Note 2 (qp-note2)"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["list"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("qp-digest2 [P] Digest Level 2"));
    assert!(!stdout.contains("qp-note1"));
    assert!(!stdout.contains("qp-note2"));
    assert!(!stdout.contains("qp-note3"));
    assert!(!stdout.contains("qp-digest1"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["list", "--no-resolve-compaction"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("qp-digest2 [P] Digest Level 2"));
    assert!(stdout.contains("qp-digest1 [P] Digest Level 1"));
    assert!(stdout.contains("qp-note1 [P] Note 1"));
    assert!(stdout.contains("qp-note2 [P] Note 2"));
    assert!(stdout.contains("qp-note3 [P] Note 3"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["show", "qp-note1"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("id: qp-digest2"));
    assert!(stdout.contains("title: Digest Level 2"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["show", "qp-note1", "--no-resolve-compaction"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("id: qp-note1"));
    assert!(stdout.contains("title: Note 1"));

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "compact",
            "show",
            "qp-digest2",
            "--compaction-depth",
            "3",
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
    assert!(compacts.contains(&serde_json::json!("qp-digest1")));
    assert!(compacts.contains(&serde_json::json!("qp-note3")));
    assert_eq!(json["depth"], 3);
    assert!(json["tree"].is_array());
    let tree = json["tree"].as_array().unwrap();
    assert!(!tree.is_empty());

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "compact",
            "show",
            "qp-digest2",
            "--compaction-depth",
            "3",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.show"));
    assert!(stdout.contains("digest=qp-digest2"));
    assert!(stdout.contains("count=2"));
    assert!(stdout.contains("D compacted qp-digest1"));
    assert!(stdout.contains("D compacted qp-note3"));
}
