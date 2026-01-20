use crate::lib::error::{QipuError, Result};
use rusqlite::{params, Connection};

impl super::super::Database {
    #[allow(dead_code)]
    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM edges WHERE source_id = ?1 OR target_id = ?1",
                params![note_id],
            )
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete edges for note {}: {}",
                    note_id, e
                ))
            })?;

        self.conn
            .execute(
                "DELETE FROM unresolved WHERE source_id = ?1",
                params![note_id],
            )
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete unresolved refs for note {}: {}",
                    note_id, e
                ))
            })?;

        self.conn
            .execute("DELETE FROM tags WHERE note_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!("failed to delete tags for note {}: {}", note_id, e))
            })?;

        let deleted_rows = self
            .conn
            .execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to delete note {}: {}", note_id, e)))?;

        if deleted_rows == 0 {
            return Err(QipuError::NoteNotFound {
                id: note_id.to_string(),
            });
        }

        Ok(())
    }

    pub(crate) fn delete_note_internal(conn: &Connection, note_id: &str) -> Result<()> {
        conn.execute("DELETE FROM edges WHERE source_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete edges for note {}: {}",
                    note_id, e
                ))
            })?;

        conn.execute("DELETE FROM edges WHERE target_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete backlinks for note {}: {}",
                    note_id, e
                ))
            })?;

        conn.execute(
            "DELETE FROM unresolved WHERE source_id = ?1",
            params![note_id],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to delete unresolved for note {}: {}",
                note_id, e
            ))
        })?;

        conn.execute("DELETE FROM tags WHERE note_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!("failed to delete tags for note {}: {}", note_id, e))
            })?;

        conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to delete note {}: {}", note_id, e)))?;

        Ok(())
    }
}
