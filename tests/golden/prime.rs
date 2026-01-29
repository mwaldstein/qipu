use crate::golden::common::{assert_golden_output, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_prime_empty_store() {
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
            .arg("prime")
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

    let golden_path = Path::new("tests/golden/prime_empty.txt");
    assert_golden_output(&normalized_output, golden_path).unwrap();
}
