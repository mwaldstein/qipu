use crate::golden::common::{assert_golden_output, create_golden_test_store, qipu};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_golden_context_with_note() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("context")
            .arg("--note")
            .arg("qp-a1b2c3")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let store_placeholder = "<STORE_PATH>";
    let normalized_output = output.replace(
        &format!("{}", store_dir.path().display()),
        store_placeholder,
    );

    let golden_path = Path::new("tests/golden/context_with_note.txt");
    assert_golden_output(&normalized_output, golden_path).unwrap();
}

#[test]
fn test_golden_context_with_moc() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("context")
            .arg("--moc")
            .arg("qp-moc123")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let store_placeholder = "<STORE_PATH>";
    let normalized_output = output.replace(
        &format!("{}", store_dir.path().display()),
        store_placeholder,
    );

    let golden_path = Path::new("tests/golden/context_with_moc.txt");
    assert_golden_output(&normalized_output, golden_path).unwrap();
}
