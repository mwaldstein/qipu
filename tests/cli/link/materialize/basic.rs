use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_link_materialize_wiki_links() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index to recognize the inline link
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize the inline link
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Materialized links"))
        .stdout(predicate::str::contains(&target_id));

    // Verify the link is now in frontmatter
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &source_id, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&target_id))
        .stdout(predicate::str::contains("related"));
}

#[test]
fn test_link_materialize_dry_run() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index to recognize the inline link
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Dry-run materialize
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id, "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[DRY RUN]"))
        .stdout(predicate::str::contains(&target_id));

    // Verify the link is NOT in frontmatter (dry run should not modify)
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &source_id, "--typed-only"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No links found").or(predicate::str::contains("typed").not()),
        );
}

#[test]
fn test_link_materialize_custom_type() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index to recognize the inline link
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize with custom type
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("supports"));

    // Verify the link has the custom type
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &source_id, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&target_id))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_link_materialize_no_inline_links() {
    let dir = setup_test_dir();

    // Create source note WITHOUT inline links
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output);

    // Index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Try to materialize (should report no-op)
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No inline links found"));
}

#[test]
fn test_link_materialize_skips_duplicates() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Create note with typed link in frontmatter and wiki link in body
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\nlinks:\n  - type: related\n    id: {}\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize should skip the duplicate
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped"))
        .stdout(predicate::str::contains(&target_id));
}

#[test]
fn test_link_materialize_remove_inline() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize and remove inline links
    qipu()
        .current_dir(dir.path())
        .args(["link", "materialize", &source_id, "--remove-inline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed inline links from body"));

    // Verify the note body was cleaned up
    let updated_content = std::fs::read_to_string(&note_path).unwrap();
    assert!(!updated_content.contains("[["));
    assert!(updated_content.contains("See") && updated_content.contains("for details"));
}

#[test]
fn test_link_materialize_json_output() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize with JSON output
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "materialize", &source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\":\"materialized\""))
        .stdout(predicate::str::contains(&target_id))
        .stdout(predicate::str::contains(&source_id));
}

#[test]
fn test_link_materialize_records_output() {
    let dir = setup_test_dir();

    // Create target note
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let target_id = extract_id(&output1);

    // Create source note with wiki link
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note", "--type", "permanent"])
        .output()
        .unwrap();
    let source_id = extract_id(&output2);

    // Update the source note to add inline wiki link
    let note_path = dir
        .path()
        .join(".qipu")
        .join("notes")
        .join(format!("{}-source-note.md", source_id));
    let content = format!(
        "---\nid: {}\ntitle: Source Note\ntype: permanent\n---\n\nSee [[{}]] for details.\n",
        source_id, target_id
    );
    std::fs::write(&note_path, content).unwrap();

    // Index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Materialize with records output
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "materialize", &source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=link.materialize"))
        .stdout(predicate::str::contains("M"));
}
