//! Golden tests for deterministic outputs of key commands
//!
//! These tests ensure that critical commands produce stable, expected outputs.
//! Golden files should be updated only when intentional output changes are made.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// Get a Command for qipu
fn qipu() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_qipu"));
    cmd.env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"));
    cmd
}

/// Create a test store with sample data for consistent testing
fn create_golden_test_store(store_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize store
    qipu().arg("--store").arg(store_dir).arg("init").output()?;

    // Create sample notes with known IDs and content for deterministic output
    let notes = vec![
        ("qp-a1b2c3", "Research Note", "permanent", "This is a permanent research note about algorithms.\n\n## Summary\n\nImportant findings about algorithmic complexity."),
        ("qp-d4e5f6", "Quick Idea", "fleeting", "Quick idea about performance optimization needs more research."),
        ("qp-g7h8i9", "Paper Review", "literature", "## Sources\n\n- https://example.com/paper\n\nReview of important research paper."),
    ];

    for (id, title, note_type, content) in notes {
        let note_content = format!("---\nid: {}\ntitle: {}\ntype: {}\ncreated: 2026-01-12T13:00:00Z\nupdated: 2026-01-12T13:00:00Z\n---\n\n{}", 
            id, title, note_type, content);

        let note_path = store_dir
            .join("notes")
            .join(format!("{}-{}.md", id, slug::slugify(title)));
        fs::create_dir_all(note_path.parent().unwrap())?;
        fs::write(note_path, note_content)?;
    }

    // Create a MOC
    let moc_content = "---\nid: qp-moc123\ntitle: Research MOC\ntype: moc\ncreated: 2026-01-12T13:00:00Z\nupdated: 2026-01-12T13:00:00Z\n---\n\n# Research Map of Content\n\n## Algorithm Research\n\n- [[qp-a1b2c3]] - Research Note\n\n## Literature\n\n- [[qp-g7h8i9]] - Paper Review\n\n## Ideas\n\n- [[qp-d4e5f6]] - Quick Idea";

    let moc_path = store_dir.join("mocs").join("qp-moc123-research-moc.md");
    fs::create_dir_all(moc_path.parent().unwrap())?;
    fs::write(moc_path, moc_content)?;

    // Build index
    qipu().arg("--store").arg(store_dir).arg("index").output()?;

    Ok(())
}

/// Assert output matches golden file
fn assert_golden_output(
    actual: &str,
    golden_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if golden_path.exists() {
        let expected = fs::read_to_string(golden_path)?;
        if actual != expected {
            eprintln!("Golden test failed!");
            eprintln!("Expected:\n{}", expected);
            eprintln!("Actual:\n{}", actual);
            eprintln!("Golden file: {}", golden_path.display());
            eprintln!(
                "To update golden file: cp /tmp/actual_output {}",
                golden_path.display()
            );

            // Write actual output to temp file for easy updating
            fs::write("/tmp/actual_output", actual)?;
            panic!(
                "Golden test output does not match {}",
                golden_path.display()
            );
        }
    } else {
        // Create golden file if it doesn't exist
        fs::create_dir_all(golden_path.parent().unwrap())?;
        fs::write(golden_path, actual)?;
        panic!(
            "Created new golden file at {}. Please verify contents and commit.",
            golden_path.display()
        );
    }
    Ok(())
}

// ============================================================================
// Help and Version Golden Tests
// ============================================================================

#[test]
fn test_golden_help_output() {
    let output = String::from_utf8(qipu().arg("--help").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/help.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_version_output() {
    let output = String::from_utf8(qipu().arg("--version").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/version.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// List Command Golden Tests
// ============================================================================

#[test]
fn test_golden_list_empty() {
    let store_dir = tempdir().unwrap();

    // Create empty store
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
            .arg("list")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/list_empty.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_list_with_notes() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/list_with_notes.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// Prime Command Golden Tests
// ============================================================================

#[test]
fn test_golden_prime_empty_store() {
    let store_dir = tempdir().unwrap();

    // Create empty store
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

// ============================================================================
// Context Command Golden Tests
// ============================================================================

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

    // Normalize store path
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

    // Normalize store path
    let store_placeholder = "<STORE_PATH>";
    let normalized_output = output.replace(
        &format!("{}", store_dir.path().display()),
        store_placeholder,
    );

    let golden_path = Path::new("tests/golden/context_with_moc.txt");
    assert_golden_output(&normalized_output, golden_path).unwrap();
}

// ============================================================================
// Search Command Golden Tests
// ============================================================================

#[test]
fn test_golden_search_basic() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("search")
            .arg("algorithms")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/search_basic.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// Inbox Command Golden Tests
// ============================================================================

#[test]
fn test_golden_inbox() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("inbox")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/inbox.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// Show Command Golden Tests
// ============================================================================

#[test]
fn test_golden_show_note() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("show")
            .arg("qp-a1b2c3")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/show_note.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_show_note_with_links() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("show")
            .arg("qp-a1b2c3")
            .arg("--links")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/show_note_with_links.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// Link Traversal Golden Tests
// ============================================================================

#[test]
fn test_golden_link_list() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("list")
            .arg("qp-moc123")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_list.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_link_tree() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("tree")
            .arg("qp-moc123")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_tree.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_link_path() {
    let store_dir = tempdir().unwrap();
    create_golden_test_store(store_dir.path()).unwrap();

    let output = String::from_utf8(
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("link")
            .arg("path")
            .arg("qp-moc123")
            .arg("qp-d4e5f6")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let golden_path = Path::new("tests/golden/link_path.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

// ============================================================================
// Error Output Golden Tests
// ============================================================================

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
