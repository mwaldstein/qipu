use crate::cli::support::{extract_id, qipu};
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
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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

[graph.types.recommends]
inverse = "recommended-by"
description = "This note recommends another note"

[graph.types."recommended-by"]
inverse = "recommends"
description = "This note is recommended by another note"
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create two notes
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
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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

    // Reopen database connection to see the latest state
    let db_path = dir.path().join(".qipu/qipu.db");
    let conn = Connection::open(db_path).unwrap();

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

#[test]
fn test_standard_type_part_of() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes: chapter (part) and book (whole)
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Chapter 1"])
        .output()
        .unwrap();
    let chapter = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Book"])
        .output()
        .unwrap();
    let book = extract_id(&output2);

    // Add part-of link: Chapter -> Book
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &chapter, &book, "--type", "part-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows part-of
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &chapter])
        .assert()
        .success()
        .stdout(predicate::str::contains(&book))
        .stdout(predicate::str::contains("part-of"));

    // Verify inverse link shows has-part
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &book])
        .assert()
        .success()
        .stdout(predicate::str::contains(&chapter))
        .stdout(predicate::str::contains("has-part"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_follows() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two sequential notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Step 2"])
        .output()
        .unwrap();
    let step2 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Step 1"])
        .output()
        .unwrap();
    let step1 = extract_id(&output2);

    // Add follows link: Step 2 follows Step 1
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &step2, &step1, "--type", "follows"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows follows
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &step2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&step1))
        .stdout(predicate::str::contains("follows"));

    // Verify inverse link shows precedes
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &step1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&step2))
        .stdout(predicate::str::contains("precedes"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_contradicts() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two contradicting notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Claim A"])
        .output()
        .unwrap();
    let claim_a = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Claim B"])
        .output()
        .unwrap();
    let claim_b = extract_id(&output2);

    // Add contradicts link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &claim_a, &claim_b, "--type", "contradicts"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows contradicts
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &claim_a])
        .assert()
        .success()
        .stdout(predicate::str::contains(&claim_b))
        .stdout(predicate::str::contains("contradicts"));

    // Verify inverse link shows contradicted-by
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &claim_b])
        .assert()
        .success()
        .stdout(predicate::str::contains(&claim_a))
        .stdout(predicate::str::contains("contradicted-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_answers() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create answer and question notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Answer Note"])
        .output()
        .unwrap();
    let answer = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Question Note"])
        .output()
        .unwrap();
    let question = extract_id(&output2);

    // Add answers link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &answer, &question, "--type", "answers"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows answers
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &answer])
        .assert()
        .success()
        .stdout(predicate::str::contains(&question))
        .stdout(predicate::str::contains("answers"));

    // Verify inverse link shows answered-by
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &question])
        .assert()
        .success()
        .stdout(predicate::str::contains(&answer))
        .stdout(predicate::str::contains("answered-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_refines() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two version notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Version 2"])
        .output()
        .unwrap();
    let v2 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Version 1"])
        .output()
        .unwrap();
    let v1 = extract_id(&output2);

    // Add refines link: V2 refines V1
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &v2, &v1, "--type", "refines"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows refines
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &v2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&v1))
        .stdout(predicate::str::contains("refines"));

    // Verify inverse link shows refined-by
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &v1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&v2))
        .stdout(predicate::str::contains("refined-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_same_as() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two synonym notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Concept A"])
        .output()
        .unwrap();
    let concept_a = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Concept B"])
        .output()
        .unwrap();
    let concept_b = extract_id(&output2);

    // Add same-as link (symmetric)
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &concept_a, &concept_b, "--type", "same-as"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows same-as
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &concept_a])
        .assert()
        .success()
        .stdout(predicate::str::contains(&concept_b))
        .stdout(predicate::str::contains("same-as"));

    // Verify inverse link also shows same-as (symmetric)
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &concept_b])
        .assert()
        .success()
        .stdout(predicate::str::contains(&concept_a))
        .stdout(predicate::str::contains("same-as"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_alias_of() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create alias and canonical notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Alternative Name"])
        .output()
        .unwrap();
    let alias = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Canonical Name"])
        .output()
        .unwrap();
    let canonical = extract_id(&output2);

    // Add alias-of link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &alias, &canonical, "--type", "alias-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows alias-of
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &alias])
        .assert()
        .success()
        .stdout(predicate::str::contains(&canonical))
        .stdout(predicate::str::contains("alias-of"));

    // Verify inverse link shows has-alias
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &canonical])
        .assert()
        .success()
        .stdout(predicate::str::contains(&alias))
        .stdout(predicate::str::contains("has-alias"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_unknown_type_fallback_inversion() {
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
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Add unknown type link (not in standard ontology or config)
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "custom-unknown"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Verify forward link shows custom-unknown
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("custom-unknown"));

    // Verify inverse link shows fallback pattern: inverse-custom-unknown
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("inverse-custom-unknown"))
        .stdout(predicate::str::contains("(virtual)"));
}
