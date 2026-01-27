use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_pack_unpack_json_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note with all fields
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Test Note")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Find the note ID from the output
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
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
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = qipu();
    cmd.arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
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
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Test Note Records")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Find the note ID
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
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
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = qipu();
    cmd.arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();
}

#[test]
fn test_typed_links_preserved_through_dump_load_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create multiple notes
    let output_a = qipu()
        .arg("create")
        .arg("Note A")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let id_a = extract_id_from_bytes(&output_a.stdout);

    let output_b = qipu()
        .arg("create")
        .arg("Note B")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let id_b = extract_id_from_bytes(&output_b.stdout);

    let output_c = qipu()
        .arg("create")
        .arg("Note C")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let id_c = extract_id_from_bytes(&output_c.stdout);

    let output_d = qipu()
        .arg("create")
        .arg("Note D")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let id_d = extract_id_from_bytes(&output_d.stdout);

    // 3. Add typed links between notes
    qipu()
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .args(["link", "add", &id_a, &id_c, "--type", "derived-from"])
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .args(["link", "add", &id_b, &id_d, "--type", "contradicts"])
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .args(["link", "add", &id_c, &id_d, "--type", "part-of"])
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Index to update the database
    qipu()
        .arg("index")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Pack to JSON
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 5. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Load into store 2
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Index the loaded notes
    qipu()
        .arg("index")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Verify typed links are preserved in store 2
    let output_a_show = qipu()
        .args([
            "show",
            &id_a,
            "--links",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let json_a: serde_json::Value = serde_json::from_slice(&output_a_show.stdout).unwrap();
    let links_a = json_a["links"].as_array().unwrap();

    // Verify Note A has links to B (supports) and C (derived-from)
    assert_eq!(links_a.len(), 2, "Note A should have 2 outbound links");

    let mut found_supports = false;
    let mut found_derived_from = false;

    for link in links_a {
        let link_id = link["id"].as_str().unwrap();
        let link_type = link["type"].as_str().unwrap();

        if link_id == id_b && link_type == "supports" {
            found_supports = true;
        }
        if link_id == id_c && link_type == "derived-from" {
            found_derived_from = true;
        }
    }

    assert!(
        found_supports,
        "Note A should have 'supports' link to Note B"
    );
    assert!(
        found_derived_from,
        "Note A should have 'derived-from' link to Note C"
    );

    let output_b_show = qipu()
        .args([
            "show",
            &id_b,
            "--links",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let json_b: serde_json::Value = serde_json::from_slice(&output_b_show.stdout).unwrap();
    let links_b = json_b["links"].as_array().unwrap();

    let mut found_contradicts = false;
    for link in links_b {
        let link_id = link["id"].as_str().unwrap();
        let link_type = link["type"].as_str().unwrap();

        if link_id == id_d && link_type == "contradicts" {
            found_contradicts = true;
        }
    }
    assert!(
        found_contradicts,
        "Note B should have 'contradicts' link to Note D"
    );

    let output_c_show = qipu()
        .args([
            "show",
            &id_c,
            "--links",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let json_c: serde_json::Value = serde_json::from_slice(&output_c_show.stdout).unwrap();
    let links_c = json_c["links"].as_array().unwrap();

    let mut found_part_of = false;
    for link in links_c {
        let link_id = link["id"].as_str().unwrap();
        let link_type = link["type"].as_str().unwrap();

        if link_id == id_d && link_type == "part-of" {
            found_part_of = true;
        }
    }
    assert!(found_part_of, "Note C should have 'part-of' link to Note D");
}

fn extract_id_from_bytes(bytes: &[u8]) -> String {
    let output = String::from_utf8_lossy(bytes);
    output
        .lines()
        .find(|line| line.starts_with("qp-"))
        .map(|line| line.trim().to_string())
        .expect("Failed to extract ID from output")
}
