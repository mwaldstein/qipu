use crate::cli::support::qipu;
use predicates::prelude::*;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn test_link_add_and_list() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links from source should show outbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("supports"));

    // List links from target should show inbound link as virtual inverted edge by default
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supported-by"))
        .stdout(predicate::str::contains("(virtual)"));

    // List links from target with --no-semantic-inversion should show raw inbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--no-semantic-inversion"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supports"))
        .stdout(predicate::str::contains("<-"));
}

#[test]
fn test_link_add_idempotent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Adding the same link again should report unchanged
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_link_remove() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success();

    // Remove the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_custom_link_inversion() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with inversion
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[links.inverses]
recommends = "recommended-by"
"recommended-by" = "recommends"

[links.descriptions]
recommends = "This note recommends another note"
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    eprintln!("Created notes: {} -> {}", id1, id2);

    // Add custom link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "recommends"])
        .assert()
        .success();

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links from target should show custom inverted edge
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("recommended-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_link_add_remove_updates_database() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    // Verify link appears in database
    let db_path = dir.path().join(".qipu/qipu.db");
    let conn = Connection::open(db_path).unwrap();

    // Debug: check all edges in database after add
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

    // Use show command to check note content before remove
    let show_output_before = qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .output()
        .unwrap();
    eprintln!(
        "Note content before remove:\n{}",
        String::from_utf8_lossy(&show_output_before.stdout)
    );

    // Remove the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    // Use show command to check note content after remove
    let show_output_after = qipu()
        .current_dir(dir.path())
        .args(["show", &id1])
        .output()
        .unwrap();
    eprintln!(
        "Note content after remove:\n{}",
        String::from_utf8_lossy(&show_output_after.stdout)
    );

    // Debug: check all edges in database
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

    // Verify link is removed from database
    let mut stmt2 = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges WHERE source_id = ?1 AND target_id = ?2")
        .unwrap();
    let mut rows = stmt2.query((&id1, &id2)).unwrap();
    assert!(
        rows.next().unwrap().is_none(),
        "Link should not exist in edges table after remove"
    );

    // Check note file content before remove
    let note_path = std::fs::read_dir(dir.path())
        .unwrap()
        .flat_map(|e| {
            std::fs::read_dir(e.unwrap().path())
                .ok()
                .into_iter()
                .flatten()
        })
        .find(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(&id1)
        })
        .unwrap()
        .unwrap()
        .path();
    let note_content = std::fs::read_to_string(&note_path).unwrap();
    eprintln!("Note content before remove:\n{}", note_content);

    // Remove the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "supports"])
        .assert()
        .success();

    // Check note file content after remove
    let note_content_after = std::fs::read_to_string(&note_path).unwrap();
    eprintln!("Note content after remove:\n{}", note_content_after);
    eprintln!("Content changed: {}", note_content != note_content_after);

    // Debug: check all edges in database
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

    // Verify link is removed from database
    let mut stmt2 = conn
        .prepare("SELECT source_id, target_id, link_type FROM edges WHERE source_id = ?1 AND target_id = ?2")
        .unwrap();
    let mut rows = stmt2.query((&id1, &id2)).unwrap();
    assert!(
        rows.next().unwrap().is_none(),
        "Link should not exist in edges table after remove"
    );
}
