use crate::lib::error::{QipuError, Result};
use crate::lib::index::types::{Edge, LinkSource};
use crate::lib::note::{LinkType, Note};
use rusqlite::{params, Connection};
use std::path::Path;

impl super::Database {
    pub fn insert_edges(&self, note: &Note) -> Result<()> {
        use crate::lib::index::links;
        use std::collections::{HashMap, HashSet};

        let mut unresolved = HashSet::new();
        let path_to_id = HashMap::new();

        if note.path.is_some() {
            let parent_path = note.path.as_ref().unwrap().parent().unwrap();

            if let Ok(existing_ids) = crate::lib::store::Store::discover(parent_path) {
                let ids = existing_ids.existing_ids().unwrap_or_default();
                let edges = links::extract_links(
                    note,
                    &ids,
                    &mut unresolved,
                    note.path.as_deref(),
                    &path_to_id,
                );

                // Delete all existing edges for this note before inserting new ones
                self.conn
                    .execute("DELETE FROM edges WHERE source_id = ?1", params![note.id()])
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to delete edges for note {}: {}",
                            note.id(),
                            e
                        ))
                    })?;

                for (position, edge) in edges.iter().enumerate() {
                    let link_type_str = edge.link_type.to_string();
                    let inline_flag =
                        if matches!(edge.source, crate::lib::index::types::LinkSource::Inline) {
                            1
                        } else {
                            0
                        };
                    let position = position as i64;

                    self.conn
                        .execute(
                            "INSERT INTO edges (source_id, target_id, link_type, inline, position) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![edge.from, edge.to, link_type_str, inline_flag, position],
                        )
                        .map_err(|e| {
                            QipuError::Other(format!("failed to insert edge {} -> {}: {}", edge.from, edge.to, e))
                        })?;
                }

                // Force WAL checkpoint to ensure changes are written to disk
                let _ = self.conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");

                // Delete all existing unresolved references for this note
                self.conn
                    .execute(
                        "DELETE FROM unresolved WHERE source_id = ?1",
                        params![note.id()],
                    )
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to delete unresolved refs for note {}: {}",
                            note.id(),
                            e
                        ))
                    })?;

                for unresolved_ref in unresolved {
                    self.conn
                        .execute(
                            "INSERT INTO unresolved (source_id, target_ref) VALUES (?1, ?2)",
                            params![note.id(), unresolved_ref],
                        )
                        .map_err(|e| {
                            QipuError::Other(format!(
                                "failed to insert unresolved ref '{}' for note {}: {}",
                                unresolved_ref,
                                note.id(),
                                e
                            ))
                        })?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn insert_edges_internal(
        conn: &Connection,
        note: &Note,
        _store_root: &Path,
    ) -> Result<()> {
        use crate::lib::index::links;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut unresolved = HashSet::new();
        let path_to_id = HashMap::new();

        if note.path.is_some() {
            let parent_path = note.path.as_ref().unwrap().parent().unwrap();

            if let Ok(existing_ids) = crate::lib::store::Store::discover(parent_path) {
                let ids = existing_ids.existing_ids().unwrap_or_default();
                let edges = links::extract_links(
                    note,
                    &ids,
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
                    let inline_flag =
                        if matches!(edge.source, crate::lib::index::types::LinkSource::Inline) {
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
                for target_ref in unresolved {
                    conn.execute(
                        "INSERT OR IGNORE INTO unresolved (source_id, target_ref) VALUES (?1, ?2)",
                        params![note.id(), target_ref],
                    )
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
        }

        Ok(())
    }

    pub fn get_backlinks(&self, note_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, link_type, inline FROM edges WHERE target_id = ?1")
            .map_err(|e| QipuError::Other(format!("failed to prepare backlinks query: {}", e)))?;

        let mut rows = stmt
            .query(params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to execute backlinks query: {}", e)))?;

        let mut backlinks = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read backlink: {}", e)))?
        {
            let source_id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get source_id: {}", e)))?;
            let link_type_str: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get link_type: {}", e)))?;
            let inline: i64 = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get inline: {}", e)))?;

            let link_type = LinkType::from(link_type_str);
            let source = if inline == 1 {
                LinkSource::Inline
            } else {
                LinkSource::Typed
            };

            backlinks.push(Edge {
                from: source_id,
                to: note_id.to_string(),
                link_type,
                source,
            });
        }

        Ok(backlinks)
    }

    /// Get outbound edges from a note (links FROM this note)
    pub fn get_outbound_edges(&self, note_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT target_id, link_type, inline FROM edges WHERE source_id = ?1 ORDER BY position")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare outbound edges query: {}", e))
            })?;

        let mut rows = stmt.query(params![note_id]).map_err(|e| {
            QipuError::Other(format!("failed to execute outbound edges query: {}", e))
        })?;

        let mut edges = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read outbound edge: {}", e)))?
        {
            let target_id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get target_id: {}", e)))?;
            let link_type_str: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get link_type: {}", e)))?;
            let inline: i64 = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get inline: {}", e)))?;

            let link_type = LinkType::from(link_type_str);
            let source = if inline == 1 {
                LinkSource::Inline
            } else {
                LinkSource::Typed
            };

            edges.push(Edge {
                from: note_id.to_string(),
                to: target_id,
                link_type,
                source,
            });
        }

        Ok(edges)
    }
}
