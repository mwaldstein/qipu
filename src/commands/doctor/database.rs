use super::types::{DoctorResult, Issue, Severity};
use crate::lib::store::Store;

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

pub fn check_broken_links(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_broken_links() {
        Ok(broken_links) => {
            for (source_id, target_ref) in broken_links {
                let path = match db.get_note_metadata(&source_id) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

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

#[allow(dead_code)]
pub fn check_orphaned_notes(store: &Store, result: &mut DoctorResult) {
    let db = store.db();

    match db.get_orphaned_notes() {
        Ok(orphaned) => {
            for note_id in orphaned {
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
