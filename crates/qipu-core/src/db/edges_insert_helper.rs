use crate::error::{QipuError, Result};
use crate::note::Note;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Options for edges insertion behavior
#[derive(Default)]
pub struct EdgeInsertOptions {
    /// Force WAL checkpoint after insertion
    pub checkpoint: bool,
    /// Use INSERT OR IGNORE for unresolved refs (vs INSERT)
    pub ignore_duplicates: bool,
}

/// Build path to ID map from existing notes
fn build_path_to_id_map(conn: &Connection) -> Result<HashMap<PathBuf, String>> {
    let mut stmt = conn
        .prepare("SELECT id, path FROM notes")
        .map_err(|e| QipuError::Other(format!("failed to prepare path query: {}", e)))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| QipuError::Other(format!("failed to execute path query: {}", e)))?;

    let mut path_to_id = HashMap::new();

    while let Some(row) = rows
        .next()
        .map_err(|e| QipuError::Other(format!("failed to read path: {}", e)))?
    {
        let id: String = row
            .get(0)
            .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
        let path: String = row
            .get(1)
            .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;

        let path_buf = PathBuf::from(path);
        path_to_id.insert(path_buf, id);
    }

    Ok(path_to_id)
}

/// Insert edges for a note using provided connection
pub fn insert_edges_with_options(
    conn: &Connection,
    note: &Note,
    existing_ids: &HashSet<String>,
    options: EdgeInsertOptions,
) -> Result<()> {
    use crate::index::links;

    let mut unresolved = HashSet::new();
    let path_to_id = build_path_to_id_map(conn)?;

    if note.path.is_some() {
        let edges = links::extract_links(
            note,
            existing_ids,
            &mut unresolved,
            note.path.as_deref(),
            &path_to_id,
        );

        // Delete all existing edges for this note before inserting new ones
        conn.execute("DELETE FROM edges WHERE source_id = ?1", params![note.id()])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete edges for note {}: {}",
                    note.id(),
                    e
                ))
            })?;

        for (position, edge) in edges.iter().enumerate() {
            let link_type_str = edge.link_type.to_string();
            let inline_flag = if matches!(edge.source, crate::index::types::LinkSource::Inline) {
                1
            } else {
                0
            };
            let position = position as i64;

            conn.execute(
                "INSERT INTO edges (source_id, target_id, link_type, inline, position) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![edge.from, edge.to, link_type_str, inline_flag, position],
            )
            .map_err(|e| {
                QipuError::Other(format!("failed to insert edge {} -> {}: {}", edge.from, edge.to, e))
            })?;
        }

        // Force WAL checkpoint if requested
        if options.checkpoint {
            let _ = conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");
        }

        // Delete all existing unresolved references for this note
        conn.execute(
            "DELETE FROM unresolved WHERE source_id = ?1",
            params![note.id()],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to delete unresolved for note {}: {}",
                note.id(),
                e
            ))
        })?;

        // Insert unresolved references
        let unresolved_sql = if options.ignore_duplicates {
            "INSERT OR IGNORE INTO unresolved (source_id, target_ref) VALUES (?1, ?2)"
        } else {
            "INSERT INTO unresolved (source_id, target_ref) VALUES (?1, ?2)"
        };

        for target_ref in unresolved {
            conn.execute(unresolved_sql, params![note.id(), target_ref])
                .map_err(|e| {
                    QipuError::Other(format!(
                        "failed to insert unresolved {} -> {}: {}",
                        note.id(),
                        target_ref,
                        e
                    ))
                })?;
        }
    }

    Ok(())
}
