use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_init_stealth_inside_existing_store_fails() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path().join(".qipu"))
        .args(["init", "--stealth"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "refusing to initialize a nested stealth store",
        ));

    assert!(
        !dir.path().join(".qipu").join(".qipu").exists(),
        "init --stealth inside a store must not create a nested store"
    );
}
