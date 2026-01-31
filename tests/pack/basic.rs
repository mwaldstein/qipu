//! Pack/unpack basic functionality tests
//!
//! Tests for packing and unpacking notes using JSON and Records formats,
//! including preservation of typed links and metadata.

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

use crate::pack::support::{create_link_with_env, create_note_with_env, qipu_store, TwoStoreSetup};

#[test]
fn test_pack_unpack_json_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    qipu_store(store1_path).arg("init").assert().success();

    // 2. Create a note with all fields
    qipu_store(store1_path)
        .arg("create")
        .arg("Test Note")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .assert()
        .success();

    // Find the note ID from the output
    let output = qipu_store(store1_path)
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // Find the note file and inject high-fidelity fields
    for entry in walkdir::WalkDir::new(store1_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                let updated_content = content.replace(
                    "tags: [tag1, tag2]",
                    "tags: [tag1, tag2]\nsummary: \"Test summary\"\ncompacts: [comp1, comp2]\nsource: \"Test source\"\nauthor: \"Test author\"\ngenerated_by: \"Test generator\"\nprompt_hash: \"Test hash\"\nverified: true"
                );
                fs::write(entry.path(), updated_content).unwrap();
                break;
            }
        }
    }

    // 3. Pack to JSON
    qipu_store(store1_path)
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // 4. Initialize store 2
    qipu_store(store2_path).arg("init").assert().success();

    // 5. Unpack/Load into store 2
    qipu_store(store2_path)
        .arg("load")
        .arg(&pack_file)
        .assert()
        .success();

    // 6. Verify note in store 2
    qipu_store(store2_path)
        .arg("show")
        .arg(&note_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"));
}

#[test]
fn test_pack_unpack_records_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.records");

    // 1. Initialize store 1
    qipu_store(store1_path).arg("init").assert().success();

    // 2. Create a note
    qipu_store(store1_path)
        .arg("create")
        .arg("Test Note Records")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .assert()
        .success();

    // Find the note ID
    let output = qipu_store(store1_path)
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // Injected fields
    for entry in walkdir::WalkDir::new(store1_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                let updated_content = content.replace(
                    "tags: [tag1, tag2]",
                    "tags: [tag1, tag2]\nsummary: \"Test summary records\"\ncompacts: [comp1, comp2]\nsource: \"Test source records\"\nauthor: \"Test author records\"\ngenerated_by: \"Test generator records\"\nprompt_hash: \"Test hash records\"\nverified: false"
                );
                fs::write(entry.path(), updated_content).unwrap();
                break;
            }
        }
    }

    // 3. Pack to Records
    qipu_store(store1_path)
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .assert()
        .success();

    // 4. Initialize store 2
    qipu_store(store2_path).arg("init").assert().success();

    // 5. Unpack/Load into store 2
    qipu_store(store2_path)
        .arg("load")
        .arg(&pack_file)
        .assert()
        .success();

    // 6. Verify note in store 2
    qipu_store(store2_path)
        .arg("show")
        .arg(&note_id)
        .assert()
        .success();
}

fn dump_to_json(stores: &TwoStoreSetup) {
    qipu_store(stores.store1_path())
        .arg("dump")
        .arg("--output")
        .arg(stores.pack_file())
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

fn load_from_json(stores: &TwoStoreSetup) {
    qipu_store(stores.store2_path())
        .arg("load")
        .arg(stores.pack_file())
        .assert()
        .success();
}

fn get_note_links_json(store_path: &std::path::Path, note_id: &str) -> serde_json::Value {
    let output = qipu_store(store_path)
        .args([
            "show",
            note_id,
            "--links",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn verify_link(links: &[serde_json::Value], target_id: &str, link_type: &str) -> bool {
    links.iter().any(|link| {
        link["id"].as_str() == Some(target_id) && link["type"].as_str() == Some(link_type)
    })
}

struct LinkGraph {
    pub id_a: String,
    pub id_b: String,
    pub id_c: String,
    pub id_d: String,
}

fn setup_link_graph(stores: &TwoStoreSetup) -> LinkGraph {
    let id_a = create_note_with_env(stores.store1_path(), "Note A");
    let id_b = create_note_with_env(stores.store1_path(), "Note B");
    let id_c = create_note_with_env(stores.store1_path(), "Note C");
    let id_d = create_note_with_env(stores.store1_path(), "Note D");

    create_link_with_env(stores.store1_path(), &id_a, &id_b, "supports");
    create_link_with_env(stores.store1_path(), &id_a, &id_c, "derived-from");
    create_link_with_env(stores.store1_path(), &id_b, &id_d, "contradicts");
    create_link_with_env(stores.store1_path(), &id_c, &id_d, "part-of");

    qipu_store(stores.store1_path())
        .arg("index")
        .assert()
        .success();

    dump_to_json(stores);
    load_from_json(stores);
    qipu_store(stores.store2_path())
        .arg("index")
        .assert()
        .success();

    LinkGraph {
        id_a,
        id_b,
        id_c,
        id_d,
    }
}

#[test]
fn test_typed_links_preserved_note_a_links() {
    let stores = TwoStoreSetup::new("test.pack.json");
    let graph = setup_link_graph(&stores);

    let json = get_note_links_json(stores.store2_path(), &graph.id_a);
    let links = json["links"].as_array().unwrap();

    assert_eq!(links.len(), 2, "Note A should have 2 outbound links");
    assert!(
        verify_link(links, &graph.id_b, "supports"),
        "Note A should have 'supports' link to Note B"
    );
    assert!(
        verify_link(links, &graph.id_c, "derived-from"),
        "Note A should have 'derived-from' link to Note C"
    );
}

#[test]
fn test_typed_links_preserved_note_b_links() {
    let stores = TwoStoreSetup::new("test.pack.json");
    let graph = setup_link_graph(&stores);

    let json = get_note_links_json(stores.store2_path(), &graph.id_b);
    let links = json["links"].as_array().unwrap();

    assert!(
        verify_link(links, &graph.id_d, "contradicts"),
        "Note B should have 'contradicts' link to Note D"
    );
}

#[test]
fn test_typed_links_preserved_note_c_links() {
    let stores = TwoStoreSetup::new("test.pack.json");
    let graph = setup_link_graph(&stores);

    let json = get_note_links_json(stores.store2_path(), &graph.id_c);
    let links = json["links"].as_array().unwrap();

    assert!(
        verify_link(links, &graph.id_d, "part-of"),
        "Note C should have 'part-of' link to Note D"
    );
}
