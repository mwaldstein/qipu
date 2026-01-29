use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_store_stats() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("store")
            .arg("stats")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let store_placeholder = "<STORE_PATH>";
    let normalized_output = output.replace(
        &format!("Store: {}", store_dir.path().display()),
        &format!("Store: {}", store_placeholder),
    );

    let golden_path = Path::new("tests/golden/store_stats.txt");
    assert_golden_output(&normalized_output, golden_path).unwrap();
}
