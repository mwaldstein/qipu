use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_merge_notes_basic() {
    let dir = tempdir().unwrap();

    // 1. Init
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // 2. Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "tag1", "Note One"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "tag2", "Note Two"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // 3. Create a third note that links to id1
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    // 4. Merge id1 into id2
    qipu()
        .current_dir(dir.path())
        .args(["merge", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Merge complete. {} has been merged into {}.",
            id1, id2
        )));

    // 5. Verify id1 is gone
    qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .assert()
        .failure();

    // 6. Verify id2 has combined tags
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tag1\""))
        .stdout(predicate::str::contains("\"tag2\""));
}
