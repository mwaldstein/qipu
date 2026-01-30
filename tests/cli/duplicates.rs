use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_doctor_duplicates_threshold() {
    use std::fs;

    let dir = setup_test_dir();

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

    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.99"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-note1"))
        .stdout(predicate::str::contains("qp-note2"));
}

#[test]
fn test_doctor_duplicates_ignores_stop_words() {
    use std::fs;

    let dir = setup_test_dir();

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
    use std::fs;

    let dir = setup_test_dir();

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
    use std::fs;

    let dir = setup_test_dir();

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

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));
}

#[test]
fn test_doctor_duplicates_stop_word_list_coverage() {
    use std::fs;

    let dir = setup_test_dir();

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
    use std::fs;

    let dir = setup_test_dir();

    let note1_content = r#"---
id: qp-field1
title: Distributed Systems Architecture
---
the and or with in for at"#;

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
    use std::fs;

    let dir = setup_test_dir();

    let note1_content = r#"---
id: qp-weight1
title: The Quantum Computing
tags: []
---
This is a basic note about computing systems."#;

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

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.8"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store is healthy"));

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--duplicates", "--threshold", "0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("near-duplicate"))
        .stdout(predicate::str::contains("qp-weight1"))
        .stdout(predicate::str::contains("qp-weight2"));
}
