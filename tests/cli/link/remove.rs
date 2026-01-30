use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use rusqlite::Connection;

#[test]
fn test_link_remove() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed link"));

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_add_remove_updates_database() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    let db_path = dir.path().join(".qipu/qipu.db");
    let conn = Connection::open(db_path).unwrap();

    let mut debug_stmt = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges ORDER BY source_id")
        .unwrap();
    let mut debug_rows = debug_stmt.query([]).unwrap();
    eprintln!("Edges in database after add:");
    while let Some(row) = debug_rows.next().unwrap() {
        let source: String = row.get(0).unwrap();
        let target: String = row.get(1).unwrap();
        let link_type: String = row.get(2).unwrap();
        eprintln!("  {} -> {} ({})", source, target, link_type);
    }

    let mut stmt = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges WHERE source_id = ?1 AND target_id = ?2")
        .unwrap();
    let mut rows = stmt.query((&id1, &id2)).unwrap();

    assert!(
        rows.next().unwrap().is_some(),
        "Link should exist in edges table after add"
    );

    let show_output_before = qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .output()
        .unwrap();
    eprintln!(
        "Note content before remove:\n{}",
        String::from_utf8_lossy(&show_output_before.stdout)
    );

    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    let show_output_after = qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .output()
        .unwrap();
    eprintln!(
        "Note content after remove:\n{}",
        String::from_utf8_lossy(&show_output_after.stdout)
    );

    let db_path = dir.path().join(".qipu/qipu.db");
    let conn = Connection::open(db_path).unwrap();

    let mut debug_stmt = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges ORDER BY source_id")
        .unwrap();
    let mut debug_rows = debug_stmt.query([]).unwrap();
    eprintln!("Edges in database after remove:");
    while let Some(row) = debug_rows.next().unwrap() {
        let source: String = row.get(0).unwrap();
        let target: String = row.get(1).unwrap();
        let link_type: String = row.get(2).unwrap();
        eprintln!("  {} -> {} ({})", source, target, link_type);
    }

    let mut stmt2 = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges WHERE source_id = ?1 AND target_id = ?2")
        .unwrap();
    let mut rows = stmt2.query((&id1, &id2)).unwrap();
    assert!(
        rows.next().unwrap().is_none(),
        "Link should not exist in edges table after remove"
    );
}
