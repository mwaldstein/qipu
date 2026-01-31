use crate::error::{QipuError, Result};
use crate::index::types::{Edge, LinkSource};
use crate::note::{LinkType, Note};
use rusqlite::{params, Connection};
use std::collections::HashSet;

use super::edges_insert_helper::{insert_edges_with_options, EdgeInsertOptions};

impl super::Database {
    pub fn insert_edges(&self, note: &Note, existing_ids: &HashSet<String>) -> Result<()> {
        insert_edges_with_options(
            &self.conn,
            note,
            existing_ids,
            EdgeInsertOptions {
                checkpoint: true,
                ignore_duplicates: false,
            },
        )
    }

    pub(crate) fn insert_edges_internal(
        conn: &Connection,
        note: &Note,
        ids: &HashSet<String>,
    ) -> Result<()> {
        insert_edges_with_options(
            conn,
            note,
            ids,
            EdgeInsertOptions {
                checkpoint: false,
                ignore_duplicates: true,
            },
        )
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
