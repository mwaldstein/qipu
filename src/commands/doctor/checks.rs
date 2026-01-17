use super::types::{DoctorResult, Issue, Severity};
use crate::lib::compaction::CompactionContext;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::similarity::SimilarityEngine;
use crate::lib::store::Store;
use std::collections::{HashMap, HashSet};
use std::fs;
use walkdir::WalkDir;

/// Check store directory structure
pub fn check_store_structure(store: &Store, result: &mut DoctorResult) {
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
pub fn scan_notes(store: &Store) -> (Vec<Note>, Vec<(String, String)>) {
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
pub fn check_duplicate_ids(notes: &[Note], result: &mut DoctorResult) {
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
pub fn check_broken_links(notes: &[Note], valid_ids: &HashSet<String>, result: &mut DoctorResult) {
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
pub fn check_required_fields(notes: &[Note], result: &mut DoctorResult) {
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
pub fn check_compaction_invariants(notes: &[Note], result: &mut DoctorResult) {
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

/// Check for near-duplicate notes using similarity
pub fn check_near_duplicates(index: &Index, threshold: f64, result: &mut DoctorResult) {
    let engine = SimilarityEngine::new(index);
    let duplicates = engine.find_all_duplicates(threshold);

    for (id1, id2, score) in duplicates {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "near-duplicate".to_string(),
            message: format!(
                "Notes '{}' and '{}' are similar ({:.2}%)",
                id1,
                id2,
                score * 100.0
            ),
            note_id: Some(id1),
            path: None,
            fixable: false, // Requires manual merge/compaction
        });
    }
}
