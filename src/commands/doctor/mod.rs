//! Doctor command - validate store invariants and optionally repair issues
//!
//! Per spec (specs/cli-interface.md):
//! - Validates store structure and note integrity
//! - Reports duplicate IDs, broken links, invalid frontmatter
//! - `--fix` auto-repairs issues where possible

pub mod checks;
pub mod fix;
pub mod report;
pub mod types;

use std::collections::HashSet;

use crate::cli::Cli;
use crate::lib::error::{QipuError, Result};
use crate::lib::store::Store;
pub use types::{DoctorResult, Issue, Severity};

/// Execute the doctor command and return the result
pub fn execute(cli: &Cli, store: &Store, fix: bool) -> Result<DoctorResult> {
    let mut result = DoctorResult::new();

    // 1. Check store structure
    checks::check_store_structure(store, &mut result);

    // 2. Scan all notes and check for issues
    let (notes, parse_errors) = checks::scan_notes(store);
    result.notes_scanned = notes.len() + parse_errors.len();

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

    // 3. Check for duplicate IDs
    checks::check_duplicate_ids(&notes, &mut result);

    // 4. Build index to check links
    let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

    // 5. Check for broken links (unresolved references)
    checks::check_broken_links(&notes, &all_ids, &mut result);

    // 6. Check for required frontmatter fields
    checks::check_required_fields(&notes, &mut result);

    // 7. Check compaction invariants
    checks::check_compaction_invariants(&notes, &mut result);

    // 8. If fix requested, attempt repairs
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
        store.create_note("Test Note", None, &[]).unwrap();

        let mut result = DoctorResult::new();
        checks::check_store_structure(&store, &mut result);

        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_doctor_duplicate_ids() {
        use crate::lib::note::NoteFrontmatter;

        let mut note1 = Note::new(
            NoteFrontmatter::new("qp-abc1".to_string(), "Note 1".to_string()),
            "Body 1".to_string(),
        );
        note1.path = Some("/path/to/note1.md".into());

        let mut note2 = Note::new(
            NoteFrontmatter::new("qp-abc1".to_string(), "Note 2".to_string()),
            "Body 2".to_string(),
        );
        note2.path = Some("/path/to/note2.md".into());

        let notes = vec![note1, note2];
        let mut result = DoctorResult::new();
        checks::check_duplicate_ids(&notes, &mut result);

        assert_eq!(result.error_count, 1);
        assert_eq!(result.issues[0].category, "duplicate-id");
    }

    #[test]
    fn test_doctor_broken_links() {
        use crate::lib::note::{LinkType, NoteFrontmatter, TypedLink};

        let mut note = Note::new(
            NoteFrontmatter::new("qp-abc1".to_string(), "Test Note".to_string()),
            "See [[qp-missing]]".to_string(),
        );
        note.frontmatter.links = vec![TypedLink {
            link_type: LinkType::Related,
            id: "qp-also-missing".to_string(),
        }];
        note.path = Some("/path/to/note.md".into());

        let valid_ids: HashSet<_> = ["qp-abc1".to_string()].into_iter().collect();
        let mut result = DoctorResult::new();
        checks::check_broken_links(&[note], &valid_ids, &mut result);

        // Should find both broken links
        assert!(result.error_count >= 1); // Typed link is an error
        assert!(result.warning_count >= 1); // Inline link is a warning
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
