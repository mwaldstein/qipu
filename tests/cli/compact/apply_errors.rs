use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_compact_apply_no_sources_error() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
A digest."#;

    std::fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", "qp-digest"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no source note IDs provided (use --note, --from-stdin, or --notes-file)",
        ));
}
