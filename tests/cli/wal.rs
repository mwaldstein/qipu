use crate::support::{extract_id_from_bytes, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_wal_concurrent_read_after_write() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let id = extract_id_from_bytes(
        &qipu()
            .current_dir(dir.path())
            .args(["create", "Note One"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );

    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note One"));
}

#[test]
fn test_wal_multiple_rapid_creates_then_list() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let ids = vec![
        extract_id_from_bytes(
            &qipu()
                .current_dir(dir.path())
                .args(["create", "Rapid One"])
                .assert()
                .success()
                .get_output()
                .stdout,
        ),
        extract_id_from_bytes(
            &qipu()
                .current_dir(dir.path())
                .args(["create", "Rapid Two"])
                .assert()
                .success()
                .get_output()
                .stdout,
        ),
        extract_id_from_bytes(
            &qipu()
                .current_dir(dir.path())
                .args(["create", "Rapid Three"])
                .assert()
                .success()
                .get_output()
                .stdout,
        ),
    ];

    qipu()
        .current_dir(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rapid One"))
        .stdout(predicate::str::contains("Rapid Two"))
        .stdout(predicate::str::contains("Rapid Three"));

    for id in ids {
        qipu()
            .current_dir(dir.path())
            .args(["show", &id])
            .assert()
            .success();
    }
}

#[test]
fn test_wal_write_then_immediate_search() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Searchable Content"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "Searchable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Searchable Content"));
}

#[test]
fn test_wal_link_create_then_traverse() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let id1 = extract_id_from_bytes(
        &qipu()
            .current_dir(dir.path())
            .args(["create", "Source Note"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );

    let id2 = extract_id_from_bytes(
        &qipu()
            .current_dir(dir.path())
            .args(["create", "Target Note"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", "--type", "related", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("related"))
        .stdout(predicate::str::contains(&id2));
}

#[test]
fn test_wal_context_after_rapid_updates() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let id = extract_id_from_bytes(
        &qipu()
            .current_dir(dir.path())
            .args(["create", "Root Note"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );

    let _child = extract_id_from_bytes(
        &qipu()
            .current_dir(dir.path())
            .args(["create", "Child Note"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );

    qipu()
        .current_dir(dir.path())
        .args(["context", "--walk", &id])
        .assert()
        .success();
}

#[test]
fn test_wal_dump_after_multiple_writes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    for i in 0..5 {
        qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Dump Note {}", i)])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .args(["dump"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dump Note 0"))
        .stdout(predicate::str::contains("Dump Note 4"));
}
