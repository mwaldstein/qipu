use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_list_empty() {
    let store_dir = tempdir().unwrap();

    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("init")
        .output()
        .unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/list_empty.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_list_with_notes() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/list_with_notes.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
