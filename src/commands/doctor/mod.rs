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
pub mod ontology;
pub mod report;
pub mod structure;
pub mod types;

use crate::cli::Cli;
use qipu_core::error::{QipuError, Result};
use qipu_core::index::IndexBuilder;
use qipu_core::store::Store;
pub use types::{DoctorResult, Issue, Severity};

/// Execute the doctor command and return the result
#[tracing::instrument(skip(cli, store), fields(store_root = %store.root().display(), fix, duplicates, threshold, check_ontology))]
pub fn execute(
    cli: &Cli,
    store: &Store,
    fix: bool,
    duplicates: bool,
    threshold: f64,
    check_ontology: bool,
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

    // 5.5. Check for semantic link type misuse (using DB)
    checks::check_semantic_link_types(store, &mut result);

    // 6. Check for required frontmatter fields
    checks::check_required_fields(&notes, &mut result);

    // 7. Check for valid value range (0-100)
    checks::check_value_range(&notes, &mut result);

    // 8. Check custom metadata
    checks::check_custom_metadata(&notes, &mut result);

    // 9. Check for missing or orphaned attachments
    checks::check_attachments(store, &notes, &mut result);

    // 10. Check compaction invariants
    checks::check_compaction_invariants(&notes, &mut result);

    // 11. Check for bare link lists (quality bar)
    checks::check_bare_link_lists(&notes, &mut result);

    // 11.5. Check for orphaned tag aliases
    checks::check_tag_aliases(store, &notes, &mut result);

    // 12. Check for overly complex notes (quality bar)
    checks::check_note_complexity(&notes, &mut result);

    // 12.5. Check for empty MOCs (MOCs with no links)
    checks::check_empty_mocs(&notes, &mut result);

    // 13. Check for near-duplicates if requested
    if duplicates {
        checks::check_near_duplicates(&index, threshold, &mut result);
    }

    // 14. Check ontology if requested
    if check_ontology {
        checks::check_ontology(store, &notes, &mut result);
    }

    // 15. If fix requested, attempt repairs
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

// Tests have been moved to their respective modules:
// - content.rs: Tests for content validation (compaction, value range, bare link lists, note complexity)
// - database.rs: Tests for database consistency (duplicate IDs, broken links, semantic link types)
// - structure.rs: Tests for store structure (currently none in mod.rs)
