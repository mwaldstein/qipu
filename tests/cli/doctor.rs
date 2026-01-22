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

// ============================================================================
// Stop-word filtering tests (end-to-end)
// Tests for specs/similarity-ranking.md stop-word filtering
// ============================================================================

#[test]
fn test_doctor_duplicates_ignores_stop_words() {
    // Test that stop words don't affect duplicate detection
    // Two notes that only differ by stop words should be detected as duplicates
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with identical content words but different stop words
    let note1_content = r#"---
id: qp-stop1
title: Knowledge Management System
---
This is a note about knowledge management and information architecture. The system provides tools for organizing notes."#;

    let note2_content = r#"---
id: qp-stop2
title: Knowledge Management System
---
This note discusses knowledge management with information architecture. System has tools to organize notes."#;

    fs::write(
        dir.path()
            .join(".qipu/notes/qp-stop1-knowledge-management.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-stop2-knowledge-management.md"),
        note2_content,
    )
    .unwrap();

    // Run doctor with --duplicates
    // Both notes should be detected as near-duplicates since stop words are filtered
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.7"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-stop1"))
        .stdout(predicate::str::contains("qp-stop2"));
}

#[test]
fn test_doctor_duplicates_stop_words_only_differences_not_detected() {
    // Test that notes differing only by stop words ARE detected as duplicates
    // This verifies stop-word filtering is working correctly
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with the same content words but tons of different stop words
    let note1_content = r#"---
id: qp-same1
title: Graph Theory
---
graph algorithms data structures computer science"#;

    let note2_content = r#"---
id: qp-same2
title: Graph Theory
---
the graph is with algorithms and for data of structures in computer on science"#;

    fs::write(
        dir.path().join(".qipu/notes/qp-same1-graph-theory.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-same2-graph-theory.md"),
        note2_content,
    )
    .unwrap();

    // Both notes should be detected as duplicates at high threshold
    // because all the stop words ("the", "is", "with", "and", "for", "of", "in", "on") are filtered
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.9"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-same1"))
        .stdout(predicate::str::contains("qp-same2"));
}

#[test]
fn test_doctor_duplicates_content_words_required_for_match() {
    // Test that actual content word differences prevent duplicate detection
    // This verifies that stop-word filtering doesn't cause false positives
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with different content words
    let note1_content = r#"---
id: qp-diff1
title: Machine Learning
---
This is a note about neural networks and deep learning algorithms for artificial intelligence."#;

    let note2_content = r#"---
id: qp-diff2
title: Database Systems
---
This is a note about relational databases and query optimization techniques for data storage."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-diff1-machine-learning.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-diff2-database-systems.md"),
        note2_content,
    )
    .unwrap();

    // These notes should NOT be detected as duplicates even at low threshold
    // because they have different content words despite similar stop-word patterns
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_duplicates_stop_word_list_coverage() {
    // Test that specific stop words from the spec are filtered
    // Per specs/similarity-ranking.md: "a", "an", "the", "and", "or", "is", "with", "in", "for", etc.
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with identical rare content words but different common stop words
    let note1_content = r#"---
id: qp-rare1
title: Zettelkasten Method
---
zettelkasten ontology epistemology methodology"#;

    let note2_content = r#"---
id: qp-rare2
title: Zettelkasten Method
---
a zettelkasten is the ontology and an epistemology with methodology or for in on at by"#;

    fs::write(
        dir.path().join(".qipu/notes/qp-rare1-zettelkasten.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-rare2-zettelkasten.md"),
        note2_content,
    )
    .unwrap();

    // Should be detected as duplicates because stop words are filtered
    // Testing stop words: "a", "is", "the", "and", "an", "with", "or", "for", "in", "on", "at", "by"
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.9"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-rare1"))
        .stdout(predicate::str::contains("qp-rare2"));
}

#[test]
fn test_doctor_duplicates_stop_words_in_title_and_body() {
    // Test that stop words are filtered from both title and body content
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Note 1: Content words in title, stop words in body
    let note1_content = r#"---
id: qp-field1
title: Distributed Systems Architecture
---
the and or with in for at"#;

    // Note 2: Same content words in title, different stop words in body
    let note2_content = r#"---
id: qp-field2
title: Distributed Systems Architecture
---
a is that this to was will"#;

    fs::write(
        dir.path().join(".qipu/notes/qp-field1-distributed.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-field2-distributed.md"),
        note2_content,
    )
    .unwrap();

    // Should be detected as duplicates because:
    // - Titles are identical (same content words)
    // - Bodies contain only stop words which are filtered
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.9"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-field1"))
        .stdout(predicate::str::contains("qp-field2"));
}

#[test]
fn test_doctor_duplicates_field_weighting_with_stop_words() {
    // Test that field weighting (title 2.0, tags 1.5, body 1.0) works correctly
    // even when stop words are mixed with content words
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Note 1: Unique term "quantum" in title (high weight)
    let note1_content = r#"---
id: qp-weight1
title: The Quantum Computing
tags: []
---
This is a basic note about computing systems."#;

    // Note 2: Same unique term "quantum" in body (low weight)
    let note2_content = r#"---
id: qp-weight2
title: Computing Systems
tags: []
---
This is a note about quantum computing and other systems."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-weight1-quantum-title.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-weight2-quantum-body.md"),
        note2_content,
    )
    .unwrap();

    // These notes share "quantum" and "computing" but in different fields
    // With stop words filtered ("the", "is", "a", "about", "and", "other"),
    // similarity should be moderate but not high enough for 0.8 threshold
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.8"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));

    // At lower threshold (0.3), they should be detected as related
    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-weight1"))
        .stdout(predicate::str::contains("qp-weight2"));
}
