use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_export_moc_selector_accepts_custom_root_type() {
    let dir = setup_test_dir();

    fs::write(
        dir.path().join(".qipu/notes/project-index-index.md"),
        "---\nid: project-index\ntitle: Project Index\ntype: outline\n---\n[Claim](claim-one-one.md)",
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/claim-one-one.md"),
        "---\nid: claim-one\ntitle: Claim One\ntype: claim\n---\nClaim body",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "project-index"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Claim One (claim-one)"))
        .stdout(predicate::str::contains("Claim body"))
        .stdout(predicate::str::contains("Project Index").not());
}

#[test]
fn test_export_outline_uses_shared_moc_selection_for_relative_markdown_links() {
    let dir = setup_test_dir();

    fs::write(
        dir.path().join(".qipu/notes/project-index-index.md"),
        "---\nid: project-index\ntitle: Project Index\ntype: outline\n---\nOutline intro\n\n[Claim](claim-one-one.md)",
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/claim-one-one.md"),
        "---\nid: claim-one\ntitle: Claim One\ntype: claim\n---\nClaim body",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "project-index", "--mode", "outline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Project Index"))
        .stdout(predicate::str::contains("Outline intro"))
        .stdout(predicate::str::contains("## Claim One (claim-one)"))
        .stdout(predicate::str::contains("Claim body"));
}
