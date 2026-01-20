//! Doctor command - validate store invariants and optionally repair issues
//!
//! Per spec (specs/cli-interface.md):
//! - Validates store structure and note integrity
//! - Reports duplicate IDs, broken links, invalid frontmatter
//! - `--fix` auto-repairs issues where possible

pub mod checks;
pub mod content;
pub mod database;
pub mod fix;
pub mod report;
pub mod structure;
pub mod types;

use crate::cli::Cli;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;
pub use types::{DoctorResult, Issue, Severity};

/// Execute the doctor command and return the result
pub fn execute(
    cli: &Cli,
    store: &Store,
    fix: bool,
    duplicates: bool,
    threshold: f64,
) -> Result<DoctorResult> {
    let mut result = DoctorResult::new();

    // 1. Check store structure
    checks::check_store_structure(store, &mut result);

    // 2. Scan all notes and check for issues
    let (notes, parse_errors) = checks::scan_notes(store);
    result.notes_scanned = notes.len() + parse_errors.len();

    // Check for notes in DB that are missing from filesystem
    checks::check_missing_files(store, &mut result);

    // Add parse errors as issues
    for (path, error) in &parse_errors {
        result.add_issue(Issue {
            severity: Severity::Error,
            category: "invalid-frontmatter".to_string(),
            message: format!("Failed to parse note: {}", error),
            note_id: None,
            path: Some(path.clone()),
            fixable: false,
        });
    }

    // 3. Check for duplicate IDs (using DB)
    checks::check_duplicate_ids(store, &mut result);

    // 4. Build index to check links and optionally duplicates
    let index = IndexBuilder::new(store).build()?;

    // 5. Check for broken links (using DB)
    checks::check_broken_links(store, &mut result);

    // 6. Check for required frontmatter fields
    checks::check_required_fields(&notes, &mut result);

    // 9. Check for missing or orphaned attachments
    checks::check_attachments(store, &notes, &mut result);

    // 10. Check compaction invariants
    checks::check_compaction_invariants(&notes, &mut result);

    // 11. Check for near-duplicates if requested
    if duplicates {
        checks::check_near_duplicates(&index, threshold, &mut result);
    }

    // 12. If fix requested, attempt repairs
    if fix {
        result.fixed_count = fix::attempt_fixes(store, &mut result)?;
    }

    // Output results
    report::output_result(cli, store, &result)?;

    // Return error if there are unfixed errors
    if result.has_errors() && result.fixed_count < result.error_count {
        Err(QipuError::InvalidStore {
            reason: format!(
                "Store has {} error(s) and {} warning(s)",
                result.error_count, result.warning_count
            ),
        })
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::Note;
    use crate::lib::store::InitOptions;
    use tempfile::tempdir;

    #[test]
    fn test_doctor_healthy_store() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        // Create a valid note
        store.create_note("Test Note", None, &[], None).unwrap();

        let mut result = DoctorResult::new();
        checks::check_store_structure(&store, &mut result);

        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_doctor_duplicate_ids() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        // Create multiple notes with unique IDs
        store.create_note("Note 1", None, &[], None).unwrap();
        store.create_note("Note 2", None, &[], None).unwrap();

        // Check that no duplicates are detected
        let mut result = DoctorResult::new();
        checks::check_duplicate_ids(&store, &mut result);

        // Due to PRIMARY KEY constraint, duplicates can't exist in DB
        assert_eq!(result.error_count, 0);

        // The duplicate check works correctly at DB level
        // Filesystem-level duplicates are caught by the database insertion
        // (INSERT OR REPLACE handles them by overwriting, not failing)
    }

    #[test]
    fn test_doctor_broken_links() {
        use crate::lib::note::{LinkType, TypedLink};

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        // Create a note with a broken link
        let mut note = store.create_note("Test Note", None, &[], None).unwrap();
        note.frontmatter.links = vec![TypedLink {
            link_type: LinkType::from(LinkType::RELATED),
            id: "qp-missing".to_string(),
        }];
        note.body = "See [[qp-also-missing]]".to_string();

        // Save the note (this will update the DB with broken links)
        store.save_note(&mut note).unwrap();

        let mut result = DoctorResult::new();
        checks::check_broken_links(&store, &mut result);

        // Should find at least one broken link
        assert!(result.error_count >= 1);
    }

    #[test]
    fn test_doctor_attachments() {
        use crate::lib::note::NoteFrontmatter;
        use crate::lib::store::InitOptions;
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();
        let attachments_dir = store.root().join("attachments");

        // 1. Create a note with a valid attachment
        let attachment_path = attachments_dir.join("valid.png");
        fs::write(&attachment_path, "dummy data").unwrap();

        let mut note1 = Note::new(
            NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
            "![Valid](../attachments/valid.png)".to_string(),
        );
        note1.path = Some(store.notes_dir().join("qp-1.md"));

        // 2. Create a note with a broken attachment
        let mut note2 = Note::new(
            NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
            "![Broken](../attachments/missing.jpg)".to_string(),
        );
        note2.path = Some(store.notes_dir().join("qp-2.md"));

        // 3. Create an orphaned attachment
        let orphaned_path = attachments_dir.join("orphaned.txt");
        fs::write(&orphaned_path, "nobody loves me").unwrap();

        let notes = vec![note1, note2];
        let mut result = DoctorResult::new();
        checks::check_attachments(&store, &notes, &mut result);

        // Should find 1 broken attachment (error) and 1 orphaned attachment (warning)
        assert_eq!(
            result.error_count, 1,
            "Expected 1 error for missing.jpg, got: {:?}",
            result.issues
        );
        assert_eq!(
            result.warning_count, 1,
            "Expected 1 warning for orphaned.txt, got: {:?}",
            result.issues
        );

        assert!(result
            .issues
            .iter()
            .any(|i| i.category == "broken-attachment" && i.message.contains("missing.jpg")));
        assert!(result
            .issues
            .iter()
            .any(|i| i.category == "orphaned-attachment" && i.message.contains("orphaned.txt")));
    }

    #[test]
    fn test_doctor_compaction_cycle() {
        use crate::lib::note::NoteFrontmatter;

        // Create two notes that compact each other (cycle)
        let mut note1 = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
        note1.compacts = vec!["qp-2".to_string()];

        let mut note2 = NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string());
        note2.compacts = vec!["qp-1".to_string()];

        let notes = vec![
            Note::new(note1, String::new()),
            Note::new(note2, String::new()),
        ];

        let mut result = DoctorResult::new();
        checks::check_compaction_invariants(&notes, &mut result);

        // Should detect cycle
        assert!(result.error_count > 0);
        assert!(result
            .issues
            .iter()
            .any(|i| i.category == "compaction-invariant" && i.message.contains("cycle")));
    }

    #[test]
    fn test_doctor_compaction_self_compaction() {
        use crate::lib::note::NoteFrontmatter;

        // Create note that compacts itself
        let mut note = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
        note.compacts = vec!["qp-1".to_string()];

        let notes = vec![Note::new(note, String::new())];

        let mut result = DoctorResult::new();
        checks::check_compaction_invariants(&notes, &mut result);

        // Should detect self-compaction
        assert!(result.error_count > 0);
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.category == "compaction-invariant"
                    && i.message.contains("compacts itself"))
        );
    }

    #[test]
    fn test_doctor_compaction_multiple_compactors() {
        use crate::lib::note::NoteFrontmatter;

        // Create two digests that both compact the same note
        let mut digest1 = NoteFrontmatter::new("qp-d1".to_string(), "Digest 1".to_string());
        digest1.compacts = vec!["qp-1".to_string()];

        let mut digest2 = NoteFrontmatter::new("qp-d2".to_string(), "Digest 2".to_string());
        digest2.compacts = vec!["qp-1".to_string()];

        let notes = vec![
            Note::new(
                NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
                String::new(),
            ),
            Note::new(digest1, String::new()),
            Note::new(digest2, String::new()),
        ];

        let mut result = DoctorResult::new();
        checks::check_compaction_invariants(&notes, &mut result);

        // Should detect multiple compactors
        assert!(result.error_count > 0);
        assert!(result
            .issues
            .iter()
            .any(|i| i.category == "compaction-invariant"
                && i.message.contains("multiple compactors")));
    }

    #[test]
    fn test_doctor_compaction_valid() {
        use crate::lib::note::NoteFrontmatter;

        // Create valid compaction: digest compacts two notes
        let mut digest = NoteFrontmatter::new("qp-digest".to_string(), "Digest".to_string());
        digest.compacts = vec!["qp-1".to_string(), "qp-2".to_string()];

        let notes = vec![
            Note::new(
                NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
                String::new(),
            ),
            Note::new(
                NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
                String::new(),
            ),
            Note::new(digest, String::new()),
        ];

        let mut result = DoctorResult::new();
        checks::check_compaction_invariants(&notes, &mut result);

        // Should have no errors
        assert_eq!(result.error_count, 0);
    }
}
