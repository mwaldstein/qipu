use crate::support::{extract_id_from_bytes, qipu};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_capture_with_provenance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Provenance Test",
            "--source",
            "https://example.com",
            "--author",
            "Test Author",
            "--generated-by",
            "test-agent",
        ])
        .write_stdin("Content with provenance")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id = extract_id_from_bytes(&output);

    let note_path = dir.path().join(".qipu").join("notes");
    let note_file = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content = fs::read_to_string(note_file).unwrap();
    assert!(note_content.contains("source: https://example.com"));
    assert!(note_content.contains("author: Test Author"));
    assert!(note_content.contains("generated_by: test-agent"));
    assert!(note_content.contains("verified: false"));
}

#[test]
fn test_capture_web_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Web Capture Test 1",
            "--source",
            "https://example.com/article",
        ])
        .write_stdin("Content from web")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id1 = extract_id_from_bytes(&output1);
    let note_path = dir.path().join(".qipu").join("notes");
    let note_file1 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id1)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content1 = fs::read_to_string(note_file1).unwrap();
    assert!(note_content1.contains("source: https://example.com/article"));
    assert!(note_content1.contains("author: Qipu Clipper"));

    let output2 = qipu()
        .current_dir(dir.path())
        .args([
            "capture",
            "--title",
            "Web Capture Test 2",
            "--source",
            "https://example.com/article2",
            "--author",
            "John Doe",
        ])
        .write_stdin("Content from web with author")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id2 = extract_id_from_bytes(&output2);
    let note_file2 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id2)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content2 = fs::read_to_string(note_file2).unwrap();
    assert!(note_content2.contains("source: https://example.com/article2"));
    assert!(note_content2.contains("author: John Doe"));
    assert!(!note_content2.contains("author: Qipu Clipper"));

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Non-web Capture Test"])
        .write_stdin("Content without source")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id3 = extract_id_from_bytes(&output3);
    let note_file3 = fs::read_dir(&note_path)
        .unwrap()
        .find_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&note_id3)
            {
                Some(path)
            } else {
                None
            }
        })
        .unwrap();

    let note_content3 = fs::read_to_string(note_file3).unwrap();
    assert!(!note_content3.contains("source:"));
    assert!(!note_content3.contains("author:"));
}
