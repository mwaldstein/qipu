use super::types::{DoctorResult, Issue, Severity};
use crate::lib::store::Store;
use std::collections::{HashMap, HashSet};

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
                LinkType::PART_OF => {
                    // Check for part-of self-loop
                    if source_id == target_id {
                        let path = match db.get_note_metadata(source_id) {
                            Ok(Some(metadata)) => Some(metadata.path),
                            _ => None,
                        };

                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "semantic-link-misuse".to_string(),
                            message: format!(
                                "Note '{}' has self-referential 'part-of' link",
                                source_id
                            ),
                            note_id: Some(source_id.clone()),
                            path,
                            fixable: false,
                        });
                    }
                }
                LinkType::FOLLOWS => {
                    // Will check for cycles in a separate pass
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

    // Check for follows cycles
    check_follows_cycles(&edges, db, result);
}

fn check_follows_cycles(
    edges: &[(String, String, String)],
    db: &crate::lib::db::Database,
    result: &mut DoctorResult,
) {
    use crate::lib::note::LinkType;

    // Build adjacency list for follows links
    let mut follows_graph: HashMap<String, Vec<String>> = HashMap::new();
    for (source_id, target_id, link_type) in edges {
        if link_type == LinkType::FOLLOWS {
            follows_graph
                .entry(source_id.clone())
                .or_default()
                .push(target_id.clone());
        }
    }

    // DFS to detect cycles
    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: HashSet<String> = HashSet::new();

    for node in follows_graph.keys() {
        if !visited.contains(node) {
            if let Some(cycle) =
                dfs_cycle_detect(node, &follows_graph, &mut visited, &mut rec_stack)
            {
                let path = match db.get_note_metadata(&cycle[0]) {
                    Ok(Some(metadata)) => Some(metadata.path),
                    _ => None,
                };

                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "semantic-link-misuse".to_string(),
                    message: format!("Detected 'follows' cycle: {}", cycle.join(" -> ")),
                    note_id: Some(cycle[0].clone()),
                    path,
                    fixable: false,
                });
            }
        }
    }
}

fn dfs_cycle_detect(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> Option<Vec<String>> {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if let Some(cycle) = dfs_cycle_detect(neighbor, graph, visited, rec_stack) {
                    if cycle.is_empty() || cycle.last() == Some(&node.to_string()) {
                        let mut new_cycle = cycle.clone();
                        new_cycle.push(node.to_string());
                        return Some(new_cycle);
                    }
                    return Some(cycle);
                }
            } else if rec_stack.contains(neighbor) {
                // Found a cycle
                let mut cycle = vec![neighbor.to_string()];
                if node != neighbor {
                    cycle.push(node.to_string());
                }
                return Some(cycle);
            }
        }
    }

    rec_stack.remove(node);
    None
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
