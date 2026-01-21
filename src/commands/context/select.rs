use crate::lib::db::Database;
use crate::lib::error::Result;
use crate::lib::note::{LinkType, NoteType};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Instant;
use tracing::debug;

/// Get note IDs linked from a MOC (including the MOC itself) with their link types
/// Returns (note_id, link_type) pairs. For the MOC itself, link_type is None.
pub fn get_moc_linked_ids(
    db: &Database,
    moc_id: &str,
    transitive: bool,
) -> Result<Vec<(String, Option<LinkType>)>> {
    let start = Instant::now();

    debug!(moc_id, transitive, "get_moc_linked_ids");

    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((moc_id.to_string(), None));

    visited.insert(moc_id.to_string());
    result.push((moc_id.to_string(), None));

    while let Some((current_id, _)) = queue.pop_front() {
        // Get outbound edges from current note
        let edges = db.get_outbound_edges(&current_id)?;

        for edge in edges {
            if visited.insert(edge.to.clone()) {
                let link_type = edge.link_type.clone();
                result.push((edge.to.clone(), Some(link_type.clone())));

                // If transitive and target is a MOC, add to queue for further traversal
                if transitive {
                    if let Some(meta) = db.get_note_metadata(&edge.to)? {
                        if meta.note_type == NoteType::Moc {
                            queue.push_back((edge.to.clone(), Some(link_type)));
                        }
                    }
                }
            }
        }
    }

    debug!(
        result_count = result.len(),
        elapsed = ?start.elapsed(),
        "get_moc_linked_ids_complete"
    );

    Ok(result)
}
