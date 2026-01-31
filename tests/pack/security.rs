use assert_cmd::{cargo::cargo_bin_cmd, Command};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_malicious_attachment_path_traversal() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("malicious.pack.json");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    qipu()
        .arg("create")
        .arg("Test Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Create a malicious pack file manually with path traversal attempt
    // We'll create a pack file with an attachment that tries to write outside attachments dir
    let pack_content = r#"{
        "header": {
            "version": "1.0",
            "store_version": 1,
            "created": "2024-01-01T00:00:00Z",
            "store_path": "/fake/store",
            "notes_count": 0,
            "attachments_count": 1,
            "links_count": 0
        },
        "notes": [],
        "links": [],
        "attachments": [
            {
                "path": "attachments/test.txt",
                "name": "../../../malicious.txt",
                "data": "VGhpcyBpcyBtYWxpY2lvdXMu",
                "content_type": "text/plain"
            }
        ]
    }"#;

    fs::write(&pack_file, pack_content).unwrap();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Attempt to load malicious pack - should succeed but sanitize path
    // The fix extracts only the filename, so "malicious.txt" will be written to attachments dir
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify malicious file was NOT written outside attachments directory
    let malicious_outside = dir2.path().join("malicious.txt");
    assert!(
        !malicious_outside.exists(),
        "Malicious file should not be written outside attachments"
    );

    // 7. Verify the file was safely written to attachments directory with just the filename
    let safe_file = store2_path.join("attachments").join("malicious.txt");
    assert!(
        safe_file.exists(),
        "Attachment should be safely written to attachments directory"
    );

    let content = fs::read_to_string(&safe_file).unwrap();
    assert_eq!(content, "This is malicious.", "Content should be preserved");
}

#[test]
fn test_malicious_attachment_absolute_path() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("malicious2.pack.json");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    qipu()
        .arg("create")
        .arg("Test Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Create malicious pack with absolute path
    let absolute_path = dir2
        .path()
        .join("absolute_attack.txt")
        .to_string_lossy()
        .to_string();
    let encoded_data = general_purpose::STANDARD.encode(b"Absolute path attack");

    let pack_content = format!(
        r#"{{
        "header": {{
            "version": "1.0",
            "store_version": 1,
            "created": "2024-01-01T00:00:00Z",
            "store_path": "/fake/store",
            "notes_count": 0,
            "attachments_count": 1,
            "links_count": 0
        }},
        "notes": [],
        "links": [],
        "attachments": [
            {{
                "path": "attachments/test.txt",
                "name": "{}",
                "data": "{}",
                "content_type": "text/plain"
            }}
        ]
    }}"#,
        absolute_path, encoded_data
    );

    fs::write(&pack_file, pack_content).unwrap();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Attempt to load - should sanitize absolute path
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify file was written to attachments directory, not the absolute path
    let absolute_attempt = dir2.path().join("absolute_attack.txt");
    assert!(
        !absolute_attempt.exists(),
        "File should not be written at absolute path"
    );

    // The filename extraction should get "absolute_attack.txt" and write to attachments
    let safe_file = store2_path.join("attachments").join("absolute_attack.txt");
    assert!(
        safe_file.exists(),
        "File should be safely written to attachments directory"
    );
}

#[test]
fn test_malicious_attachment_null_bytes() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let pack_file = dir1.path().join("malicious3.pack.json");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    qipu()
        .arg("create")
        .arg("Test Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Create malicious pack with null bytes in filename
    let pack_content = r#"{
        "header": {
            "version": "1.0",
            "store_version": 1,
            "created": "2024-01-01T00:00:00Z",
            "store_path": "/fake/store",
            "notes_count": 0,
            "attachments_count": 0,
            "links_count": 0
        },
        "notes": [],
        "links": [],
        "attachments": []
    }"#;

    fs::write(&pack_file, pack_content).unwrap();
}
