use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Doctor command tests (per specs/cli-interface.md)
// ============================================================================

#[test]
fn test_doctor_healthy_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a valid note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Healthy Note"])
        .assert()
        .success();

    // Doctor should succeed with no issues
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"notes_scanned\""))
        .stdout(predicate::str::contains("\"error_count\""))
        .stdout(predicate::str::contains("\"warning_count\""))
        .stdout(predicate::str::contains("\"issues\""));
}

#[test]
fn test_doctor_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=doctor"));
}

#[test]
fn test_doctor_missing_store() {
    let dir = tempdir().unwrap();

    // No init - should fail with exit code 3
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_doctor_broken_link_detection() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note With Link"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Link note1 -> note2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Delete note2's file directly to create a broken link
    let store_path = dir.path().join(".qipu/notes");
    for entry in std::fs::read_dir(&store_path).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&id2) {
            std::fs::remove_file(entry.path()).unwrap();
            break;
        }
    }

    // Doctor should detect missing file
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .code(3)
        .stdout(predicate::str::contains("missing-file"))
        .stdout(predicate::str::contains(&id2));
}

#[test]
fn test_doctor_fix_flag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Remove the config file to create a fixable issue
    std::fs::remove_file(dir.path().join(".qipu/config.toml")).unwrap();

    // Doctor without --fix should report the issue
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success() // Warning-level issues don't cause failure
        .stdout(predicate::str::contains("missing-config"));

    // Doctor with --fix should repair
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--fix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fixed"));

    // Config should be restored
    assert!(dir.path().join(".qipu/config.toml").exists());

    // Doctor again should show no issues
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_compaction_cycle_detection() {
    use std::fs;

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
    use std::fs;

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

#[test]
fn test_doctor_duplicates_threshold() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with identical content (100% similarity)
    let note1_content = r#"---
id: qp-note1
title: Similar Note
---
This is a note about apple banana and cherry fruits and many more fruits that are delicious and healthy to eat every day."#;

    let note2_content = r#"---
id: qp-note2
title: Similar Note
---
This is a note about apple banana and cherry fruits and many more fruits that are delicious and healthy to eat every day."#;

    let note3_content = r#"---
id: qp-note3
title: Different Note
---
This is a completely different note about programming and coding."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-similar-note-one.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-similar-note-two.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note3-different-note.md"),
        note3_content,
    )
    .unwrap();

    // Run doctor without --duplicates flag - should not report duplicates
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));

    // Run doctor with --duplicates at low threshold (0.5)
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));

    // Run doctor with --duplicates at high threshold (0.99)
    // Since notes are 100% identical, they should still be detected even at 0.99
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.99"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));

    // Run doctor with --duplicates at default threshold (0.85)
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));
}
