//! Doctor command - validate store invariants and optionally repair issues
//!
//! Per spec (specs/cli-interface.md):
//! - Validates store structure and note integrity
//! - Reports duplicate IDs, broken links, invalid frontmatter
//! - `--fix` auto-repairs issues where possible

use std::collections::{HashMap, HashSet};
use std::fs;

use serde::Serialize;
use walkdir::WalkDir;

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::IndexBuilder;
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Warning - store is functional but suboptimal
    Warning,
    /// Error - store has data integrity issues
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A single diagnostic issue
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// Issue severity
    pub severity: Severity,
    /// Issue category (e.g., "duplicate-id", "broken-link", "invalid-frontmatter")
    pub category: String,
    /// Human-readable description
    pub message: String,
    /// Affected note ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<String>,
    /// Affected file path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Can this issue be auto-fixed?
    pub fixable: bool,
}

/// Result of running doctor checks
#[derive(Debug, Clone, Serialize)]
pub struct DoctorResult {
    /// Total notes scanned
    pub notes_scanned: usize,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// All issues found
    pub issues: Vec<Issue>,
    /// Issues that were fixed (when --fix is used)
    pub fixed_count: usize,
}

impl DoctorResult {
    fn new() -> Self {
        DoctorResult {
            notes_scanned: 0,
            error_count: 0,
            warning_count: 0,
            issues: Vec::new(),
            fixed_count: 0,
        }
    }

    fn add_issue(&mut self, issue: Issue) {
        match issue.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.issues.push(issue);
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

/// Execute the doctor command
pub fn execute(cli: &Cli, store: &Store, fix: bool) -> Result<()> {
    let mut result = DoctorResult::new();

    // 1. Check store structure
    check_store_structure(store, &mut result);

    // 2. Scan all notes and check for issues
    let (notes, parse_errors) = scan_notes(store);
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
    check_duplicate_ids(&notes, &mut result);

    // 4. Build index to check links
    let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

    // 5. Check for broken links (unresolved references)
    check_broken_links(&notes, &all_ids, &mut result);

    // 6. Check for required frontmatter fields
    check_required_fields(&notes, &mut result);

    // 7. Check compaction invariants
    check_compaction_invariants(&notes, &mut result);

    // 8. If fix requested, attempt repairs
    if fix {
        result.fixed_count = attempt_fixes(store, &mut result)?;
    }

    // Output results
    output_result(cli, store, &result)?;

    // Return error if there are unfixed errors
    if result.has_errors() && result.fixed_count < result.error_count {
        Err(QipuError::InvalidStore {
            reason: format!(
                "Store has {} error(s) and {} warning(s)",
                result.error_count, result.warning_count
            ),
        })
    } else {
        Ok(())
    }
}

/// Check store directory structure
fn check_store_structure(store: &Store, result: &mut DoctorResult) {
    // Check notes directory
    if !store.notes_dir().exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "Notes directory does not exist".to_string(),
            note_id: None,
            path: Some(store.notes_dir().display().to_string()),
            fixable: true,
        });
    }

    // Check mocs directory
    if !store.mocs_dir().exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "MOCs directory does not exist".to_string(),
            note_id: None,
            path: Some(store.mocs_dir().display().to_string()),
            fixable: true,
        });
    }

    // Check config file
    let config_path = store.root().join("config.toml");
    if !config_path.exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-config".to_string(),
            message: "Config file does not exist".to_string(),
            note_id: None,
            path: Some(config_path.display().to_string()),
            fixable: true,
        });
    }
}

/// Scan all notes in the store, returning both valid notes and parse errors
fn scan_notes(store: &Store) -> (Vec<Note>, Vec<(String, String)>) {
    let mut notes = Vec::new();
    let mut errors = Vec::new();

    for dir in [store.notes_dir(), store.mocs_dir()] {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                match fs::read_to_string(path) {
                    Ok(content) => match Note::parse(&content, Some(path.to_path_buf())) {
                        Ok(note) => notes.push(note),
                        Err(e) => {
                            errors.push((path.display().to_string(), e.to_string()));
                        }
                    },
                    Err(e) => {
                        errors.push((path.display().to_string(), format!("Failed to read: {}", e)));
                    }
                }
            }
        }
    }

    (notes, errors)
}

/// Check for duplicate note IDs
fn check_duplicate_ids(notes: &[Note], result: &mut DoctorResult) {
    let mut id_to_paths: HashMap<String, Vec<String>> = HashMap::new();

    for note in notes {
        let id = note.id().to_string();
        let path = note
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        id_to_paths.entry(id).or_default().push(path);
    }

    for (id, paths) in id_to_paths {
        if paths.len() > 1 {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "duplicate-id".to_string(),
                message: format!(
                    "Duplicate ID '{}' found in {} files: {}",
                    id,
                    paths.len(),
                    paths.join(", ")
                ),
                note_id: Some(id),
                path: Some(paths.join(", ")),
                fixable: false, // Requires manual resolution
            });
        }
    }
}

/// Check for broken links (references to non-existent notes)
fn check_broken_links(notes: &[Note], valid_ids: &HashSet<String>, result: &mut DoctorResult) {
    use regex::Regex;

    let wiki_link_re =
        Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").expect("Invalid wiki link regex pattern");

    for note in notes {
        let from_id = note.id().to_string();
        let path = note
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Check typed links in frontmatter
        for link in &note.frontmatter.links {
            if !valid_ids.contains(&link.id) {
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "broken-link".to_string(),
                    message: format!(
                        "Note '{}' has typed link to non-existent note '{}'",
                        from_id, link.id
                    ),
                    note_id: Some(from_id.clone()),
                    path: Some(path.clone()),
                    fixable: true, // Can remove broken link from frontmatter
                });
            }
        }

        // Check wiki links in body
        for cap in wiki_link_re.captures_iter(&note.body) {
            let to_id = cap[1].trim().to_string();
            if to_id.is_empty() {
                continue;
            }

            // Only check links that look like qipu IDs
            if to_id.starts_with("qp-") && !valid_ids.contains(&to_id) {
                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "broken-link".to_string(),
                    message: format!(
                        "Note '{}' has inline link to non-existent note '{}'",
                        from_id, to_id
                    ),
                    note_id: Some(from_id.clone()),
                    path: Some(path.clone()),
                    fixable: false, // Requires manual edit of note body
                });
            }
        }
    }
}

/// Check for required frontmatter fields
fn check_required_fields(notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        let path = note
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // ID is required
        if note.id().is_empty() {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "missing-field".to_string(),
                message: format!("Note at '{}' is missing required 'id' field", path),
                note_id: None,
                path: Some(path.clone()),
                fixable: false, // Would need to generate ID
            });
        }

        // Title is required
        if note.title().is_empty() {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "missing-field".to_string(),
                message: format!("Note '{}' is missing required 'title' field", note.id()),
                note_id: Some(note.id().to_string()),
                path: Some(path.clone()),
                fixable: false, // Would need to generate title
            });
        }
    }
}

/// Check compaction invariants
/// Per spec (specs/compaction.md):
/// - At most one compactor per note
/// - No cycles in compaction chains
/// - No self-compaction
/// - All compaction references resolve to existing notes
fn check_compaction_invariants(notes: &[Note], result: &mut DoctorResult) {
    // Build compaction context - this enforces "at most one compactor" invariant
    let compaction_ctx = match CompactionContext::build(notes) {
        Ok(ctx) => ctx,
        Err(e) => {
            // Multiple compactors error caught during build
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "compaction-invariant".to_string(),
                message: e.to_string(),
                note_id: None,
                path: None,
                fixable: false, // Requires manual resolution
            });
            return; // Can't continue validation without valid context
        }
    };

    // Validate all compaction invariants
    let errors = compaction_ctx.validate(notes);
    for error in errors {
        result.add_issue(Issue {
            severity: Severity::Error,
            category: "compaction-invariant".to_string(),
            message: error,
            note_id: None,
            path: None,
            fixable: false, // Requires manual resolution of compaction relationships
        });
    }
}

/// Attempt to fix issues that are marked as fixable
fn attempt_fixes(store: &Store, result: &mut DoctorResult) -> Result<usize> {
    let mut fixed = 0;

    for issue in &result.issues {
        if !issue.fixable {
            continue;
        }

        match issue.category.as_str() {
            "missing-directory" => {
                if let Some(path) = &issue.path {
                    if fs::create_dir_all(path).is_ok() {
                        fixed += 1;
                    }
                }
            }
            "missing-config" => {
                // Recreate default config
                let config = crate::lib::config::StoreConfig::default();
                let config_path = store.root().join("config.toml");
                if config.save(&config_path).is_ok() {
                    fixed += 1;
                }
            }
            "broken-link" => {
                // For typed links (frontmatter), we can remove the broken link
                if let Some(note_id) = &issue.note_id {
                    if let Ok(mut note) = store.get_note(note_id) {
                        // Remove broken links from frontmatter
                        let valid_ids = store.existing_ids().unwrap_or_default();
                        let original_len = note.frontmatter.links.len();
                        note.frontmatter.links.retain(|l| valid_ids.contains(&l.id));

                        if note.frontmatter.links.len() < original_len
                            && store.save_note(&mut note).is_ok()
                        {
                            fixed += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Also rebuild indexes to ensure consistency
    let index = IndexBuilder::new(store).rebuild().build()?;
    index.save(&store.root().join(".cache"))?;

    Ok(fixed)
}

/// Output the doctor result in the appropriate format
fn output_result(cli: &Cli, store: &Store, result: &DoctorResult) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Human => {
            if result.issues.is_empty() {
                if !cli.quiet {
                    println!("Store is healthy ({} notes scanned)", result.notes_scanned);
                }
            } else {
                println!(
                    "Found {} issue(s) in {} notes:",
                    result.issues.len(),
                    result.notes_scanned
                );
                println!();

                for issue in &result.issues {
                    let severity_prefix = match issue.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARN ",
                    };

                    let fixable_suffix = if issue.fixable { " [fixable]" } else { "" };

                    println!(
                        "  {} [{}] {}{}",
                        severity_prefix, issue.category, issue.message, fixable_suffix
                    );

                    if let Some(path) = &issue.path {
                        if issue.note_id.is_none() {
                            println!("         at {}", path);
                        }
                    }
                }

                println!();
                println!(
                    "Summary: {} error(s), {} warning(s)",
                    result.error_count, result.warning_count
                );

                if result.fixed_count > 0 {
                    println!("Fixed {} issue(s)", result.fixed_count);
                }
            }
        }
        OutputFormat::Records => {
            // Header
            println!(
                "H qipu=1 records=1 store={} mode=doctor notes={} errors={} warnings={}",
                store_path_for_records(store),
                result.notes_scanned,
                result.error_count,
                result.warning_count
            );

            // Issues as diagnostic lines (D prefix)
            for issue in &result.issues {
                let note_part = issue
                    .note_id
                    .as_ref()
                    .map(|id| format!(" note={}", id))
                    .unwrap_or_default();

                println!(
                    "D {} {} \"{}\"{}",
                    issue.severity, issue.category, issue.message, note_part
                );
            }
        }
    }

    Ok(())
}

/// Get store path for records output (helper to work around borrow issues)
fn store_path_for_records(store: &Store) -> String {
    store.root().display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::store::InitOptions;
    use tempfile::tempdir;

    #[test]
    fn test_doctor_healthy_store() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        // Create a valid note
        store.create_note("Test Note", None, &[]).unwrap();

        let mut result = DoctorResult::new();
        check_store_structure(&store, &mut result);

        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_doctor_duplicate_ids() {
        // This test would require manually creating files with duplicate IDs
        // For now, test the detection logic with mock data
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
        check_duplicate_ids(&notes, &mut result);

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
        check_broken_links(&[note], &valid_ids, &mut result);

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
        check_compaction_invariants(&notes, &mut result);

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
        check_compaction_invariants(&notes, &mut result);

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
        check_compaction_invariants(&notes, &mut result);

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
        check_compaction_invariants(&notes, &mut result);

        // Should have no errors
        assert_eq!(result.error_count, 0);
    }
}
