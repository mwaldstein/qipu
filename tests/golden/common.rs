use std::fs;
use std::path::Path;
use std::process::Command;

pub fn qipu() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_qipu"));
    cmd.env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"));
    cmd
}

pub fn create_golden_test_store(store_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    qipu().arg("--store").arg(store_dir).arg("init").output()?;

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

    let moc_content = "---\nid: qp-moc123\ntitle: Research MOC\ntype: moc\ncreated: 2026-01-12T13:00:00Z\nupdated: 2026-01-12T13:00:00Z\n---\n\n# Research Map of Content\n\n## Algorithm Research\n\n- [[qp-a1b2c3]] - Research Note\n\n## Literature\n\n- [[qp-g7h8i9]] - Paper Review\n\n## Ideas\n\n- [[qp-d4e5f6]] - Quick Idea";

    let moc_path = store_dir.join("mocs").join("qp-moc123-research-moc.md");
    fs::create_dir_all(moc_path.parent().unwrap())?;
    fs::write(moc_path, moc_content)?;

    qipu().arg("--store").arg(store_dir).arg("index").output()?;

    Ok(())
}

pub fn assert_golden_output(
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

            fs::write("/tmp/actual_output", actual)?;
            panic!(
                "Golden test output does not match {}",
                golden_path.display()
            );
        }
    } else {
        fs::create_dir_all(golden_path.parent().unwrap())?;
        fs::write(golden_path, actual)?;
        panic!(
            "Created new golden file at {}. Please verify contents and commit.",
            golden_path.display()
        );
    }
    Ok(())
}
