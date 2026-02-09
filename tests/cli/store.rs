use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use predicates::prelude::*;

// ============================================================================
// Store command tests (per specs/operational-database.md)
// ============================================================================

#[test]
fn test_store_stats_empty() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["store", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store:"))
        .stdout(predicate::str::contains("Database:"))
        .stdout(predicate::str::contains("Notes: 0"))
        .stdout(predicate::str::contains("Tags: 0"))
        .stdout(predicate::str::contains("Links: 0"))
        .stdout(predicate::str::contains("Unresolved links: 0"));
}

#[test]
fn test_store_stats_with_notes() {
    let dir = setup_test_dir();

    // Create some notes and extract IDs
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1", "--tag", "test"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2", "--tag", "research"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Add a link between notes
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", "--type", "related", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["store", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Notes: 2"))
        .stdout(predicate::str::contains("Tags: 2"))
        .stdout(predicate::str::contains("Links: 1"))
        .stdout(predicate::str::contains("Unresolved links: 0"))
        .stdout(predicate::str::contains("Schema version: 9"));
}

#[test]
fn test_store_stats_json_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["store", "stats", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"database\""))
        .stdout(predicate::str::contains("\"notes\""))
        .stdout(predicate::str::contains("\"tags\""))
        .stdout(predicate::str::contains("\"links\""))
        .stdout(predicate::str::contains("\"unresolved_links\""))
        .stdout(predicate::str::contains("\"size_bytes\""))
        .stdout(predicate::str::contains("\"schema_version\""));
}

#[test]
fn test_store_stats_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["store", "stats", "--format", "records"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=stats"))
        .stdout(predicate::str::contains("database.path="))
        .stdout(predicate::str::contains("database.size="))
        .stdout(predicate::str::contains("database.schema_version="))
        .stdout(predicate::str::contains("notes="))
        .stdout(predicate::str::contains("tags="))
        .stdout(predicate::str::contains("links="))
        .stdout(predicate::str::contains("unresolved="));
}
