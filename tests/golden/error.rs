use crate::golden::common::{assert_golden_output, qipu};
use std::path::Path;

#[test]
fn test_golden_error_missing_store() {
    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg("/nonexistent/store")
            .arg("list")
            .output()
            .unwrap()
            .stderr,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/error_missing_store.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
