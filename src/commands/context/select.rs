use crate::lib::db::Database;
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use std::collections::HashSet;

/// Get note IDs linked from a MOC (including the MOC itself)
pub fn get_moc_linked_ids(db: &Database, moc_id: &str, transitive: bool) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue = vec![moc_id.to_string()];

    visited.insert(moc_id.to_string());
    result.push(moc_id.to_string());

    while let Some(current_id) = queue.pop() {
        // Get outbound edges from current note
        let edges = db.get_outbound_edges(&current_id)?;

        for edge in edges {
            if visited.insert(edge.to.clone()) {
                result.push(edge.to.clone());

                // If transitive and target is a MOC, add to queue for further traversal
                if transitive {
                    if let Some(meta) = db.get_note_metadata(&edge.to)? {
                        if meta.note_type == NoteType::Moc {
                            queue.push(edge.to.clone());
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}
