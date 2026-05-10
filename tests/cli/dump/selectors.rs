use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_dump_by_tag() {
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
        .arg("Project note")
        .arg("--id")
        .arg("note-a")
        .arg("--tag")
        .arg("project")
        .arg("--tag")
        .arg("important")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Another project note")
        .arg("--id")
        .arg("note-b")
        .arg("--tag")
        .arg("project")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Personal note")
        .arg("--id")
        .arg("note-c")
        .arg("--tag")
        .arg("personal")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--tag")
        .arg("project")
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

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_id_flag_shows_selector_guidance() {
    qipu()
        .args(["dump", "--id", "qp-a"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Use: qipu dump --note <id>"))
        .stderr(predicate::str::contains(
            "Positional FILE is the pack output path",
        ));
}

#[test]
fn test_dump_positional_existing_note_shows_output_file_guidance() {
    let dir = tempdir().unwrap();

    qipu()
        .arg("init")
        .env("QIPU_STORE", dir.path())
        .assert()
        .success();
    qipu()
        .args(["create", "--id", "note-a", "Note A"])
        .env("QIPU_STORE", dir.path())
        .assert()
        .success();

    qipu()
        .args(["dump", "note-a"])
        .env("QIPU_STORE", dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "positional argument is an output file",
        ))
        .stderr(predicate::str::contains(
            "Use: qipu dump --note note-a --output <file>",
        ));
}

#[test]
fn test_dump_by_tag_does_not_traverse_by_default() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    qipu()
        .arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Tagged note")
        .arg("--id")
        .arg("note-a")
        .arg("--tag")
        .arg("start")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Linked note")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("dump")
        .arg("--tag")
        .arg("start")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    qipu()
        .arg("load")
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

    assert_eq!(ids, vec!["note-a"]);
}

#[test]
fn test_dump_by_moc() {
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
        .arg("My MOC")
        .arg("--id")
        .arg("my-moc")
        .arg("--type")
        .arg("moc")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-a")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-b")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--moc")
        .arg("my-moc")
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
    let mut ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    ids.sort();

    eprintln!("Loaded notes: {:?}", ids);
    assert!(ids.contains(&"my-moc"), "Should contain linked root");
    assert!(ids.contains(&"note-a"), "Should contain note-a");
    assert!(ids.contains(&"note-b"), "Should contain note-b");
    assert!(
        !ids.contains(&"note-c"),
        "Should not contain note-c (not linked from MOC)"
    );
}

#[test]
fn test_dump_by_query() {
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
        .arg("Rust programming tutorial")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Advanced Rust techniques")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Python basics")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--query")
        .arg("Rust")
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

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_by_moc_follows_relative_markdown_links() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    qipu()
        .arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    std::fs::write(
        store_path.join("notes/note-alpha-alpha.md"),
        "---\nid: note-alpha\ntitle: Alpha\n---\nAlpha body",
    )
    .unwrap();
    std::fs::write(
        store_path.join("notes/note-beta-beta.md"),
        "---\nid: note-beta\ntitle: Beta\n---\nBeta body",
    )
    .unwrap();
    std::fs::write(
        store_path.join("mocs/qp-map-map.md"),
        "---\nid: qp-map\ntitle: Map\ntype: moc\n---\n[Beta](../notes/note-beta-beta.md)\n[Alpha](../notes/note-alpha-alpha.md)\n",
    )
    .unwrap();

    qipu()
        .arg("index")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("dump")
        .arg("--moc")
        .arg("qp-map")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    qipu()
        .arg("load")
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

    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"qp-map"));
    assert!(ids.contains(&"note-alpha"));
    assert!(ids.contains(&"note-beta"));
}
