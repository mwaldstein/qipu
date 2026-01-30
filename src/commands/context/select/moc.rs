use super::state::SelectionState;
use crate::commands::context::types::ContextOptions;
use qipu_core::db::Database;
use qipu_core::error::Result;
use qipu_core::note::{LinkType, Note};
use qipu_core::store::Store;
use std::collections::{HashMap, HashSet, VecDeque};
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
        let edges = db.get_outbound_edges(&current_id)?;

        for edge in edges {
            let to = edge.to.clone();
            if visited.insert(to.clone()) {
                let link_type = edge.link_type.clone();
                result.push((to.clone(), Some(link_type.clone())));

                if transitive {
                    if let Some(meta) = db.get_note_metadata(&to)? {
                        if meta.note_type.is_moc() {
                            queue.push_back((to, Some(link_type)));
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

/// Collect notes linked from a MOC
pub fn collect_from_moc<'a>(
    state: &mut SelectionState<'a>,
    store: &'a Store,
    options: &ContextOptions<'a>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    let Some(moc) = options.moc_id else {
        return Ok(());
    };

    let linked_ids = get_moc_linked_ids(store.db(), moc, options.transitive)?;
    for (id, link_type) in linked_ids {
        let resolved_id = resolve_id(&id)?;
        state.add_note(&id, resolved_id, note_map, None, link_type)?;
    }

    Ok(())
}
