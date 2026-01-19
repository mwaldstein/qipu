use super::types::{DoctorResult, Issue, Severity};
use crate::lib::compaction::CompactionContext;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::similarity::SimilarityEngine;
use crate::lib::store::paths::ATTACHMENTS_DIR;
use crate::lib::store::Store;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
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

    // Check attachments directory
    let attachments_dir = store.root().join(ATTACHMENTS_DIR);
    if !attachments_dir.exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "Attachments directory does not exist".to_string(),
            note_id: None,
            path: Some(attachments_dir.display().to_string()),
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
pub fn check_duplicate_ids(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_duplicate_ids() {
        Ok(duplicates) => {
            for (id, paths) in duplicates {
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
                    fixable: false,
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to check for duplicate IDs: {}", e);
        }
    }
}

/// Check for notes that exist in DB but not on filesystem
pub fn check_missing_files(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_missing_files() {
        Ok(missing) => {
            for (id, path) in missing {
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "missing-file".to_string(),
                    message: format!(
                        "Note '{}' exists in database but file is missing: {}",
                        id, path
                    ),
                    note_id: Some(id),
                    path: Some(path),
                    fixable: false,
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to check for missing files: {}", e);
        }
    }
}

/// Check for broken links (references to non-existent notes)
pub fn check_broken_links(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_broken_links() {
        Ok(broken_links) => {
            for (source_id, target_ref) in broken_links {
                // Get note path from DB for better error reporting
                let path = match db.get_note_metadata(&source_id) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

                // Determine severity: typed links are errors, inline links are warnings
                // The unresolved table doesn't distinguish, so we'll report all as errors
                // since typed links (from frontmatter) are auto-tracked
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "broken-link".to_string(),
                    message: format!(
                        "Note '{}' has link to non-existent note '{}'",
                        source_id, target_ref
                    ),
                    note_id: Some(source_id),
                    path,
                    fixable: true,
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to check for broken links: {}", e);
        }
    }
}

/// Check for orphaned notes (notes with no incoming links)
#[allow(dead_code)]
pub fn check_orphaned_notes(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_orphaned_notes() {
        Ok(orphaned) => {
            for note_id in orphaned {
                // Get note path from DB for better error reporting
                let path = match db.get_note_metadata(&note_id) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "orphaned-note".to_string(),
                    message: format!("Note '{}' has no incoming links", note_id),
                    note_id: Some(note_id),
                    path,
                    fixable: false,
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to check for orphaned notes: {}", e);
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

/// Check for missing or orphaned attachments
pub fn check_attachments(store: &Store, notes: &[Note], result: &mut DoctorResult) {
    let attachments_dir = store.root().join(ATTACHMENTS_DIR);
    let mut referenced_attachments = HashSet::new();

    // 1. Find all referenced attachments in notes
    // Pattern for markdown links to attachments: [label](../attachments/filename)
    // or just checking any relative path that contains "attachments/"
    let attachment_re = Regex::new(r"\[[^\]]*\]\(([^)]*attachments/[^)]+)\)")
        .expect("Invalid attachment regex pattern");

    for note in notes {
        let from_id = note.id().to_string();
        let note_path = note
            .path
            .as_ref()
            .map(|p| p.parent().unwrap_or(Path::new("")))
            .unwrap_or(Path::new(""));

        for cap in attachment_re.captures_iter(&note.body) {
            let rel_path_str = &cap[1];
            // Resolve relative path against note's location
            let full_path = note_path.join(rel_path_str);

            // Normalize path to check if it's inside our attachments directory
            if let Ok(canonical_path) = fs::canonicalize(&full_path) {
                if let Ok(canonical_attachments_dir) = fs::canonicalize(&attachments_dir) {
                    if canonical_path.starts_with(&canonical_attachments_dir) {
                        referenced_attachments.insert(canonical_path.clone());

                        // Check if the file exists
                        if !canonical_path.exists() {
                            result.add_issue(Issue {
                                severity: Severity::Error,
                                category: "broken-attachment".to_string(),
                                message: format!(
                                    "Note '{}' references missing attachment: {}",
                                    from_id, rel_path_str
                                ),
                                note_id: Some(from_id.clone()),
                                path: note.path.as_ref().map(|p| p.display().to_string()),
                                fixable: false,
                            });
                        }
                    }
                }
            } else {
                // Path resolution failed, attachment is definitely missing or invalid
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "broken-attachment".to_string(),
                    message: format!(
                        "Note '{}' references missing or invalid attachment: {}",
                        from_id, rel_path_str
                    ),
                    note_id: Some(from_id.clone()),
                    path: note.path.as_ref().map(|p| p.display().to_string()),
                    fixable: false,
                });
            }
        }
    }

    // 2. Check for orphaned attachments (files in attachments/ not referenced by any note)
    if attachments_dir.exists() {
        for entry in WalkDir::new(&attachments_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(canonical_path) = fs::canonicalize(path) {
                    if !referenced_attachments.contains(&canonical_path) {
                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "orphaned-attachment".to_string(),
                            message: format!(
                                "Orphaned attachment found: {}",
                                path.strip_prefix(&attachments_dir)
                                    .unwrap_or(path)
                                    .display()
                            ),
                            note_id: None,
                            path: Some(path.display().to_string()),
                            fixable: true, // Can be deleted
                        });
                    }
                }
            }
        }
    }
}
