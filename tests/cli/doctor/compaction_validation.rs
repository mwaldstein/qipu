use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_doctor_compaction_cycle_detection() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes that compact each other (cycle)
    let note1_content = r#"---
id: qp-note1
title: Note 1
compacts:
  - qp-note2
---
This is note 1."#;

    let note2_content = r#"---
id: qp-note2
title: Note 2
compacts:
  - qp-note1
---
This is note 2."#;

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

    // Doctor should detect the compaction cycle
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .code(3)
        .stdout(predicate::str::contains("compaction-invariant"))
        .stdout(predicate::str::contains("cycle"));
}

#[test]
fn test_doctor_compaction_multiple_compactors() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two digests that both compact the same note
    let note_content = r#"---
id: qp-source
title: Source Note
---
This is the source note."#;

    let digest1_content = r#"---
id: qp-digest1
title: Digest 1
compacts:
  - qp-source
---
This is digest 1."#;

    let digest2_content = r#"---
id: qp-digest2
title: Digest 2
compacts:
  - qp-source
---
This is digest 2."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-source-source-note.md"),
        note_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest1-digest-1.md"),
        digest1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest2-digest-2.md"),
        digest2_content,
    )
    .unwrap();

    // Doctor should detect multiple compactors
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .code(3)
        .stdout(predicate::str::contains("compaction-invariant"))
        .stdout(predicate::str::contains("multiple compactors"));
}
