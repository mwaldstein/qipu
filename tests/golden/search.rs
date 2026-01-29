use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_search_basic() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("search")
            .arg("algorithms")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/search_basic.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
