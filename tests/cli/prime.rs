use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Prime command tests (per specs/llm-context.md)
// ============================================================================

#[test]
fn test_prime_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"))
        .stdout(predicate::str::contains("Quick Reference"))
        .stdout(predicate::str::contains("qipu list"));
}

#[test]
fn test_prime_with_mocs() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Research Topics",
            "--type",
            "moc",
            "--tag",
            "research",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Key Maps of Content"))
        .stdout(predicate::str::contains("Research Topics"));
}

#[test]
fn test_prime_with_recent_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create some notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "First Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Second Note", "--type", "permanent"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recently Updated Notes"))
        .stdout(predicate::str::contains("First Note"))
        .stdout(predicate::str::contains("Second Note"));
}

#[test]
fn test_prime_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"primer\""))
        .stdout(predicate::str::contains("\"mocs\""))
        .stdout(predicate::str::contains("\"recent_notes\""))
        .stdout(predicate::str::contains("\"commands\""));
}

#[test]
fn test_prime_json_comprehensive_structure() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create MOC and notes with tags
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc", "--tag", "moc-tag"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Tagged Note",
            "--type",
            "fleeting",
            "--tag",
            "research",
            "--tag",
            "important",
        ])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify top-level structure
    assert!(json["store"].is_string(), "store should be a string");
    assert!(json["primer"].is_object(), "primer should be an object");
    assert!(json["mocs"].is_array(), "mocs should be an array");
    assert!(
        json["recent_notes"].is_array(),
        "recent_notes should be an array"
    );

    // Verify primer structure
    assert!(
        json["primer"]["description"].is_string(),
        "primer.description should be a string"
    );
    assert!(
        json["primer"]["commands"].is_array(),
        "primer.commands should be an array"
    );

    // Verify commands array has expected structure
    let commands = json["primer"]["commands"].as_array().unwrap();
    assert!(!commands.is_empty(), "commands should not be empty");
    for cmd in commands {
        assert!(cmd["name"].is_string(), "command name should be a string");
        assert!(
            cmd["description"].is_string(),
            "command description should be a string"
        );
    }

    // Verify mocs array structure
    let mocs = json["mocs"].as_array().unwrap();
    assert_eq!(mocs.len(), 1, "should have 1 MOC");
    let moc = &mocs[0];
    assert!(moc["id"].is_string(), "MOC id should be a string");
    assert!(moc["title"].is_string(), "MOC title should be a string");
    assert!(moc["tags"].is_array(), "MOC tags should be an array");
    let moc_tags = moc["tags"].as_array().unwrap();
    assert_eq!(moc_tags.len(), 1, "MOC should have 1 tag");
    assert_eq!(moc_tags[0], "moc-tag", "MOC tag should be correct");

    // Verify recent_notes array structure
    let recent_notes = json["recent_notes"].as_array().unwrap();
    assert_eq!(recent_notes.len(), 1, "should have 1 recent note");
    let note = &recent_notes[0];
    assert!(note["id"].is_string(), "note id should be a string");
    assert!(note["title"].is_string(), "note title should be a string");
    assert!(note["type"].is_string(), "note type should be a string");
    assert!(note["tags"].is_array(), "note tags should be an array");
    let note_tags = note["tags"].as_array().unwrap();
    assert_eq!(note_tags.len(), 2, "note should have 2 tags");
    let tags: Vec<&str> = note_tags.iter().map(|t| t.as_str().unwrap()).collect();
    assert!(
        tags.contains(&"research"),
        "note should have 'research' tag"
    );
    assert!(
        tags.contains(&"important"),
        "note should have 'important' tag"
    );
}

#[test]
fn test_prime_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("truncated=false"))
        .stdout(predicate::str::contains("D Qipu is"))
        .stdout(predicate::str::contains("C list"))
        .stdout(predicate::str::contains("M ")) // MOC record
        .stdout(predicate::str::contains("N ")); // Note record
}

#[test]
fn test_prime_records_comprehensive_structure() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create MOC and notes with tags
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc", "--tag", "moc-tag"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Tagged Note",
            "--type",
            "fleeting",
            "--tag",
            "research",
            "--tag",
            "important",
        ])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Verify header line
    assert!(lines[0].starts_with("H "), "First line should be header");
    assert!(lines[0].contains("qipu=1"), "Header should contain qipu=1");
    assert!(
        lines[0].contains("records=1"),
        "Header should contain records=1"
    );
    assert!(lines[0].contains("store="), "Header should contain store=");
    assert!(
        lines[0].contains("mode=prime"),
        "Header should contain mode=prime"
    );
    assert!(lines[0].contains("mocs=1"), "Header should contain mocs=1");
    assert!(
        lines[0].contains("recent=1"),
        "Header should contain recent=1"
    );
    assert!(
        lines[0].contains("truncated=false"),
        "Header should contain truncated=false"
    );

    // Verify description line (D)
    let d_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("D "))
        .copied()
        .collect();
    assert!(!d_lines.is_empty(), "Should have at least one D line");

    // Verify command lines (C)
    let c_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("C "))
        .copied()
        .collect();
    assert_eq!(c_lines.len(), 8, "Should have 8 command lines");
    for c_line in &c_lines {
        assert!(c_line.contains(" "), "Command line should have space");
    }

    // Verify MOC record (M)
    let m_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("M "))
        .copied()
        .collect();
    assert_eq!(m_lines.len(), 1, "Should have 1 MOC line");
    let m_line = &m_lines[0];
    assert!(m_line.contains("qp-"), "MOC line should contain ID");
    assert!(m_line.contains("Test MOC"), "MOC line should contain title");
    assert!(
        m_line.contains("tags=moc-tag"),
        "MOC line should have correct tag"
    );

    // Verify note record (N)
    let n_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("N "))
        .copied()
        .collect();
    assert_eq!(n_lines.len(), 1, "Should have 1 note line");
    let n_line = &n_lines[0];
    assert!(n_line.contains("qp-"), "Note line should contain ID");
    assert!(n_line.contains("fleeting"), "Note line should contain type");
    assert!(
        n_line.contains("Tagged Note"),
        "Note line should contain title"
    );
    assert!(
        n_line.contains("tags=important,research"),
        "Note line should have correct tags (comma-separated, alphabetically sorted)"
    );
}

#[test]
fn test_prime_records_empty_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create MOC and note without tags
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Verify MOC record with empty tags
    let m_line = lines.iter().find(|l| l.starts_with("M ")).unwrap();
    assert!(
        m_line.contains("tags=-"),
        "MOC line should have tags=- for empty tags"
    );

    // Verify note record with empty tags
    let n_line = lines.iter().find(|l| l.starts_with("N ")).unwrap();
    assert!(
        n_line.contains("tags=-"),
        "Note line should have tags=- for empty tags"
    );
}

#[test]
fn test_prime_records_truncated_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=false"),
        "prime records output should contain truncated=false in header"
    );
}

#[test]
fn test_prime_missing_store() {
    let dir = tempdir().unwrap();
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");

    // No init - should fail with exit code 3
    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
        .arg("prime")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}
