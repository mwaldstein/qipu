use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
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
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");

    // No init - should fail with exit code 3
    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
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

// ============================================================================
// Custom metadata tests (per specs/custom-metadata.md)
// ============================================================================

#[test]
fn test_doctor_custom_metadata_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["create", "Note Without Custom"])
        .assert()
        .success();

    // Doctor should show no issues for normal custom metadata
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_custom_metadata_normal_size() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with small custom metadata
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Small Custom"])
        .assert()
        .success()
        .get_output()
        .clone();
    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "status", "in-progress"])
        .assert()
        .success();

    // Doctor should show no issues for normal-sized custom metadata
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_custom_metadata_large() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with very large custom metadata (>10KB)
    let note_content = r#"---
id: qp-large-custom
title: Note with Large Custom
custom:
  large_data: "PLACEHOLDER"
---
This is a note."#;

    // Replace placeholder with large data (>10KB)
    let large_value = "x".repeat(12 * 1024); // 12KB
    let note_content = note_content.replace("PLACEHOLDER", &large_value);

    fs::write(
        dir.path()
            .join(".qipu/notes/qp-large-custom-note-with-large.md"),
        note_content,
    )
    .unwrap();

    // Doctor should warn about large custom metadata
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("large-custom-metadata"))
        .stdout(predicate::str::contains("qp-large-custom"));
}

#[test]
fn test_doctor_custom_metadata_multiple_notes() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes with different custom metadata sizes
    let note1_content = r#"---
id: qp-note1
title: Normal Note 1
custom:
  status: active
---
Normal note 1."#;

    let note2_content = r#"---
id: qp-note2
title: Normal Note 2
custom:
  status: inactive
  priority: 5
---
Normal note 2."#;

    // Note with large custom data
    let large_value = "data".repeat(3 * 1024); // 12KB
    let note3_content = format!(
        r#"---
id: qp-note3
title: Large Custom Note
custom:
  large_field: "{}"
---
Note with large custom field."#,
        large_value
    );

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-normal-note-1.md"),
        note1_content,
    )
    .unwrap();

    fs::write(
        dir.path().join(".qipu/notes/qp-note2-normal-note-2.md"),
        note2_content,
    )
    .unwrap();

    fs::write(
        dir.path().join(".qipu/notes/qp-note3-large-custom-note.md"),
        note3_content,
    )
    .unwrap();

    // Doctor should warn about the large custom metadata but not the normal ones
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("large-custom-metadata"))
        .stdout(predicate::str::contains("qp-note3"))
        .stdout(predicate::str::contains("qp-note1").not())
        .stdout(predicate::str::contains("qp-note2").not());
}

#[test]
fn test_doctor_ontology_invalid_note_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_content = r#"---
id: qp-note1
title: Test Note
type: invalid-type
---
Test content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-invalid-type.md"),
        note_content,
    )
    .unwrap();

    // Doctor with --check ontology should report invalid note type
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("invalid-note-type"))
        .stdout(predicate::str::contains("invalid-type"));

    // Doctor without --check ontology should not report the issue
    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("invalid-note-type").not());
}

#[test]
fn test_doctor_ontology_invalid_link_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .assert()
        .success();

    let source_note_content = r#"---
id: qp-source
title: Source Note
links:
  - type: invalid-link
    id: qp-target
---
Content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-source-source-note.md"),
        source_note_content,
    )
    .unwrap();

    // Doctor with --check ontology should report invalid link type
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("invalid-link-type"))
        .stdout(predicate::str::contains("invalid-link"));
}

#[test]
fn test_doctor_ontology_deprecated_graph_types() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_content = r#"[graph.types.custom-link]
cost = 1.5
"#;

    fs::write(dir.path().join(".qipu/config.toml"), config_content).unwrap();

    // Doctor with --check ontology should report deprecated config
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deprecated-config"))
        .stdout(predicate::str::contains("[graph.types.custom-link]"))
        .stdout(predicate::str::contains(
            "[ontology.link_types.custom-link]",
        ));
}
