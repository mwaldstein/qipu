use crate::support::qipu;
use tempfile::tempdir;

#[test]
fn test_dump_collection_root_selector_accepts_custom_root_type() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    qipu()
        .arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    std::fs::write(
        store_path.join("notes/project-index-index.md"),
        "---\nid: project-index\ntitle: Project Index\ntype: outline\n---\n[Claim](claim-one-one.md)",
    )
    .unwrap();
    std::fs::write(
        store_path.join("notes/claim-one-one.md"),
        "---\nid: claim-one\ntitle: Claim One\ntype: claim\n---\nClaim body",
    )
    .unwrap();

    qipu()
        .arg("index")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    qipu()
        .arg("dump")
        .arg("--collection-root")
        .arg("project-index")
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
    let mut ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|note| note["id"].as_str().unwrap())
        .collect();
    ids.sort();

    assert_eq!(ids, vec!["claim-one", "project-index"]);
}
