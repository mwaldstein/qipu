use crate::support::qipu;
use tempfile::tempdir;

#[test]
fn test_dump_semantic_inversion_default() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Source")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Target")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-b")
        .arg("--output")
        .arg(&pack_file)
        .arg("--direction")
        .arg("both")
        .arg("--max-hops")
        .arg("1")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert!(ids.len() >= 2, "Should include both notes");
    assert!(ids.contains(&"note-a"), "Should include source note");
    assert!(ids.contains(&"note-b"), "Should include target note");
}

#[test]
fn test_dump_semantic_inversion_disabled() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Disabled Source")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Disabled Target")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-b")
        .arg("--output")
        .arg(&pack_file)
        .arg("--direction")
        .arg("both")
        .arg("--max-hops")
        .arg("1")
        .arg("--no-semantic-inversion")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert!(ids.len() >= 2, "Should include both notes");
    assert!(ids.contains(&"note-a"), "Should include source note");
    assert!(ids.contains(&"note-b"), "Should include target note");
}

#[test]
fn test_dump_no_selectors_full_store() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .arg("--tag")
        .arg("project")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .arg("--tag")
        .arg("personal")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .arg("--type")
        .arg("moc")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note D")
        .arg("--id")
        .arg("note-d")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 4);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(ids.contains(&"note-c"));
    assert!(ids.contains(&"note-d"));

    let output = qipu()
        .arg("link")
        .arg("list")
        .arg("note-a")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let links: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let link_targets: Vec<&str> = links
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["direction"].as_str() == Some("out"))
        .map(|l| l["id"].as_str().unwrap())
        .collect();

    assert!(link_targets.contains(&"note-b"));
}
