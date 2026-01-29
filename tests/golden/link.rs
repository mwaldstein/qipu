use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_link_list() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("list")
            .arg("qp-moc123")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_list.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_link_tree() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("tree")
            .arg("qp-moc123")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_tree.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_link_path() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("path")
            .arg("qp-moc123")
            .arg("qp-d4e5f6")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_path.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
