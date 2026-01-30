//! Records format body marker tests

use crate::support::{extract_id, qipu, setup_test_dir};
use tempfile::tempdir;

#[test]
fn test_records_body_markers() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Body Markers Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Context with --with-body should include body with proper markers
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            &id,
            "--with-body",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have header and note metadata
    assert!(stdout.contains("H qipu=1 records=1"));
    assert!(stdout.contains("N "));
    assert!(stdout.contains(&id));

    // Should have B line (start of body)
    assert!(
        stdout.contains(&format!("B {}", id)),
        "Should have B line. Output: {}",
        stdout
    );

    // Should have B-END marker (end of body)
    assert!(
        stdout.contains("B-END"),
        "Should have B-END marker after B line"
    );
}
