use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_verify_note() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("verify")
            .arg("qp-a1b2c3")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/verify_note.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
