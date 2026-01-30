use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

struct PackTestSetup {
    _dir1: tempfile::TempDir,
    _dir2: tempfile::TempDir,
    pack_file: std::path::PathBuf,
}

impl PackTestSetup {
    fn new(pack_name: &str) -> Self {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        let pack_file = dir1.path().join(pack_name);

        qipu()
            .arg("init")
            .env("QIPU_STORE", dir1.path())
            .assert()
            .success();

        qipu()
            .arg("init")
            .env("QIPU_STORE", dir2.path())
            .assert()
            .success();

        Self {
            _dir1: dir1,
            _dir2: dir2,
            pack_file,
        }
    }

    fn store1_path(&self) -> &std::path::Path {
        self._dir1.path()
    }

    fn store2_path(&self) -> &std::path::Path {
        self._dir2.path()
    }
}

fn create_note_in_store(store_path: &std::path::Path, title: &str) -> String {
    qipu()
        .arg("create")
        .arg(title)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    list[0]["id"].as_str().unwrap().to_string()
}

fn find_note_file(store_path: &std::path::Path, note_id: &str) -> std::path::PathBuf {
    for entry in walkdir::WalkDir::new(store_path.join("notes")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(note_id) {
                return entry.path().to_path_buf();
            }
        }
    }
    panic!("Could not find note file for ID: {}", note_id);
}

fn add_attachment_ref(note_path: &std::path::Path, ref_text: &str) {
    let content = fs::read_to_string(note_path).unwrap();
    let updated = content.replace("## Notes\n", &format!("## Notes\n\n{}\n", ref_text));
    fs::write(note_path, updated).unwrap();
}

fn create_minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0x99, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn setup_attachments(store_path: &std::path::Path, files: &[(&str, &[u8])]) {
    let attachments_dir = store_path.join("attachments");
    fs::create_dir_all(&attachments_dir).unwrap();

    for (name, content) in files {
        fs::write(attachments_dir.join(name), *content).unwrap();
    }
}

fn rebuild_index(store_path: &std::path::Path) {
    qipu()
        .arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();
}

fn dump_pack(setup: &PackTestSetup, args: &[&str]) {
    let mut cmd_args = vec!["dump", "--output", setup.pack_file.to_str().unwrap()];
    cmd_args.extend(args);

    qipu()
        .args(&cmd_args)
        .env("QIPU_STORE", setup.store1_path())
        .assert()
        .success();
}

fn load_pack(setup: &PackTestSetup) {
    qipu()
        .arg("load")
        .arg(&setup.pack_file)
        .env("QIPU_STORE", setup.store2_path())
        .assert()
        .success();
}

fn verify_note_loaded(setup: &PackTestSetup, note_id: &str, expected_title: &str) {
    qipu()
        .arg("show")
        .arg(note_id)
        .env("QIPU_STORE", setup.store2_path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_title));
}

fn verify_attachment_restored(setup: &PackTestSetup, name: &str, expected_content: &[u8]) {
    let file = setup.store2_path().join("attachments").join(name);
    assert!(file.exists(), "Attachment {} should exist", name);
    let content = fs::read(&file).unwrap();
    assert_eq!(
        content, expected_content,
        "Attachment {} content should match",
        name
    );
}

#[test]
fn test_pack_attachments_roundtrip() {
    let setup = PackTestSetup::new("test.pack");
    let note_id = create_note_in_store(setup.store1_path(), "Note with Attachments");

    setup_attachments(
        setup.store1_path(),
        &[
            ("test1.txt", b"Test attachment 1 content"),
            ("test2.json", b"{\"key\": \"value\"}"),
            ("image.png", &create_minimal_png()),
        ],
    );

    let note_path = find_note_file(setup.store1_path(), &note_id);
    add_attachment_ref(
        &note_path,
        "See attachment: ![test1](../attachments/test1.txt)\nAnd: ![test2](../attachments/test2.json)\nImage: ![image](../attachments/image.png)",
    );

    rebuild_index(setup.store1_path());
    dump_pack(&setup, &[]);
    load_pack(&setup);

    let pack_content = fs::read_to_string(&setup.pack_file).unwrap();
    assert!(pack_content.contains("name=test1.txt"));
    assert!(pack_content.contains("name=test2.json"));
    assert!(pack_content.contains("name=image.png"));

    verify_note_loaded(&setup, &note_id, "Note with Attachments");

    verify_attachment_restored(&setup, "test1.txt", b"Test attachment 1 content");
    verify_attachment_restored(&setup, "test2.json", b"{\"key\": \"value\"}");
    verify_attachment_restored(&setup, "image.png", &create_minimal_png());
}

#[test]
fn test_pack_no_attachments_flag() {
    let setup = PackTestSetup::new("test_no_attach.pack");
    let note_id = create_note_in_store(setup.store1_path(), "Note without Attachments");

    setup_attachments(
        setup.store1_path(),
        &[("should_not_pack.txt", b"This should not be packed")],
    );

    let note_path = find_note_file(setup.store1_path(), &note_id);
    add_attachment_ref(
        &note_path,
        "See: ![file](../attachments/should_not_pack.txt)",
    );

    rebuild_index(setup.store1_path());
    dump_pack(&setup, &["--no-attachments"]);
    load_pack(&setup);

    let pack_content = fs::read_to_string(&setup.pack_file).unwrap();
    assert!(!pack_content.contains("name=should_not_pack.txt"));
    assert!(!pack_content.contains("This should not be packed"));

    verify_note_loaded(&setup, &note_id, "Note without Attachments");

    let file = setup.store2_path().join("attachments/should_not_pack.txt");
    assert!(
        !file.exists(),
        "Attachment should NOT be restored with --no-attachments"
    );
}

#[test]
fn test_pack_attachments_multiple_notes() {
    let setup = PackTestSetup::new("test_multi.pack");
    let note1_id = create_note_in_store(setup.store1_path(), "First Note");
    let note2_id = create_note_in_store(setup.store1_path(), "Second Note");

    setup_attachments(
        setup.store1_path(),
        &[
            ("shared.txt", b"Shared file"),
            ("note1_only.txt", b"Note 1 only"),
            ("note2_only.txt", b"Note 2 only"),
        ],
    );

    let note1_path = find_note_file(setup.store1_path(), &note1_id);
    add_attachment_ref(
        &note1_path,
        "![shared](../attachments/shared.txt)\n![note1](../attachments/note1_only.txt)",
    );

    let note2_path = find_note_file(setup.store1_path(), &note2_id);
    add_attachment_ref(
        &note2_path,
        "![shared](../attachments/shared.txt)\n![note2](../attachments/note2_only.txt)",
    );

    rebuild_index(setup.store1_path());
    dump_pack(&setup, &[]);
    load_pack(&setup);

    let pack_content = fs::read_to_string(&setup.pack_file).unwrap();
    assert!(pack_content.contains("name=shared.txt"));
    assert!(pack_content.contains("name=note1_only.txt"));
    assert!(pack_content.contains("name=note2_only.txt"));

    verify_attachment_restored(&setup, "shared.txt", b"Shared file");
    verify_attachment_restored(&setup, "note1_only.txt", b"Note 1 only");
    verify_attachment_restored(&setup, "note2_only.txt", b"Note 2 only");
}
