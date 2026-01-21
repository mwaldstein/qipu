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

pub fn check_semantic_link_types(store: &Store, result: &mut DoctorResult) {
    use crate::lib::note::LinkType;
    use std::collections::{HashMap, HashSet};

    let db = store.db();

    // Get all typed edges from the database
    let edges = match db.get_all_typed_edges() {
        Ok(edges) => edges,
        Err(e) => {
            tracing::error!("Failed to query edges: {}", e);
            return;
        }
    };

    // Build a map of note relationships for validation
    let mut note_relationships: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for (source_id, target_id, link_type) in &edges {
        note_relationships
            .entry(source_id.clone())
            .or_default()
            .push((target_id.clone(), link_type.clone()));
    }

    // Check for semantic misuses
    for (source_id, relationships) in &note_relationships {
        // Check for conflicting relationships
        let mut supports_targets = HashSet::new();
        let mut contradicts_targets = HashSet::new();
        let mut same_as_targets = HashSet::new();
        let mut alias_of_targets = HashSet::new();

        for (target_id, link_type) in relationships {
            match link_type.as_str() {
                LinkType::SUPPORTS => {
                    supports_targets.insert(target_id);
                }
                LinkType::CONTRADICTS => {
                    contradicts_targets.insert(target_id);
                }
                LinkType::SAME_AS => {
                    // Check for self-referential same-as
                    if source_id == target_id {
                        let path = match db.get_note_metadata(source_id) {
                            Ok(Some(metadata)) => Some(metadata.path),
                            _ => None,
                        };

                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "semantic-link-misuse".to_string(),
                            message: format!(
                                "Note '{}' has self-referential 'same-as' link",
                                source_id
                            ),
                            note_id: Some(source_id.clone()),
                            path,
                            fixable: false,
                        });
                    }
                    same_as_targets.insert(target_id);
                }
                LinkType::ALIAS_OF => {
                    // Check for self-referential alias-of
                    if source_id == target_id {
                        let path = match db.get_note_metadata(source_id) {
                            Ok(Some(metadata)) => Some(metadata.path),
                            _ => None,
                        };

                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "semantic-link-misuse".to_string(),
                            message: format!(
                                "Note '{}' has self-referential 'alias-of' link",
                                source_id
                            ),
                            note_id: Some(source_id.clone()),
                            path,
                            fixable: false,
                        });
                    }
                    alias_of_targets.insert(target_id);
                }
                _ => {}
            }
        }

        // Check for conflicting support/contradict relationships
        let conflicts: HashSet<_> = supports_targets
            .intersection(&contradicts_targets)
            .collect();

        if !conflicts.is_empty() {
            for target_id in conflicts {
                let path = match db.get_note_metadata(source_id) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "semantic-link-misuse".to_string(),
                    message: format!(
                        "Note '{}' both supports and contradicts note '{}'",
                        source_id, target_id
                    ),
                    note_id: Some(source_id.clone()),
                    path,
                    fixable: false,
                });
            }
        }

        // Check for both same-as and alias-of to same target
        let identity_conflicts: HashSet<_> =
            same_as_targets.intersection(&alias_of_targets).collect();

        if !identity_conflicts.is_empty() {
            for target_id in identity_conflicts {
                let path = match db.get_note_metadata(source_id) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "semantic-link-misuse".to_string(),
                    message: format!(
                        "Note '{}' has both 'same-as' and 'alias-of' links to note '{}'",
                        source_id, target_id
                    ),
                    note_id: Some(source_id.clone()),
                    path,
                    fixable: false,
                });
            }
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
