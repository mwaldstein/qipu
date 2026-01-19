use crate::lib::error::{QipuError, Result};
use crate::lib::index::types::NoteMetadata;
use crate::lib::note::{Note, NoteType};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::str::FromStr;

impl super::Database {
    pub fn get_note_metadata(&self, note_id: &str) -> Result<Option<NoteMetadata>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, type, path, created, updated FROM notes WHERE id = ?1")
            .map_err(|e| QipuError::Other(format!("failed to prepare query: {}", e)))?;

        let note_opt = stmt.query_row(params![note_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let path: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            Ok((id, title, note_type, path, created_dt, updated_dt))
        });

        match note_opt {
            Ok((id, title, note_type, path, created, updated)) => {
                let mut tag_stmt = self
                    .conn
                    .prepare("SELECT tag FROM tags WHERE note_id = ?1")
                    .map_err(|e| QipuError::Other(format!("failed to prepare tag query: {}", e)))?;

                let mut tags = Vec::new();
                let mut tag_rows = tag_stmt
                    .query(params![&id])
                    .map_err(|e| QipuError::Other(format!("failed to query tags: {}", e)))?;

                while let Some(row) = tag_rows
                    .next()
                    .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?
                {
                    tags.push(
                        row.get(0)
                            .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?,
                    );
                }

                Ok(Some(NoteMetadata {
                    id,
                    title,
                    note_type,
                    tags,
                    path,
                    created,
                    updated,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!(
                "failed to query note metadata: {}",
                e
            ))),
        }
    }

    pub fn insert_note(&self, note: &Note) -> Result<()> {
        let created_str = note.frontmatter.created.map(|dt| dt.to_rfc3339());
        let updated_str = note.frontmatter.updated.map(|dt| dt.to_rfc3339());
        let mtime = note
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let tags_str = note.frontmatter.tags.join(" ");

        self.conn
            .execute(
                "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    note.id(),
                    note.frontmatter.title,
                    note.frontmatter.note_type.unwrap_or(NoteType::Fleeting).to_string(),
                    note.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                    created_str,
                    updated_str,
                    &note.body,
                    mtime,
                ],
            )
            .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note.id(), e)))?;

        let rowid: i64 = self.conn.last_insert_rowid();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
                params![
                    rowid,
                    note.frontmatter.title,
                    &note.body,
                    tags_str,
                ],
            )
            .map_err(|e| QipuError::Other(format!("failed to insert note into FTS5 {}: {}", note.id(), e)))?;

        self.conn
            .execute("DELETE FROM tags WHERE note_id = ?1", params![note.id()])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete tags for note {}: {}",
                    note.id(),
                    e
                ))
            })?;

        for tag in &note.frontmatter.tags {
            self.conn
                .execute(
                    "INSERT INTO tags (note_id, tag) VALUES (?1, ?2)",
                    params![note.id(), tag],
                )
                .map_err(|e| {
                    QipuError::Other(format!(
                        "failed to insert tag {} for note {}: {}",
                        tag,
                        note.id(),
                        e
                    ))
                })?;
        }

        Ok(())
    }

    pub(super) fn insert_note_internal(conn: &Connection, note: &Note) -> Result<()> {
        let path_str = note
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or_else(|| QipuError::Other(format!("invalid path for note {}", note.id())))?;

        let created_str = note.frontmatter.created.map(|d| d.to_rfc3339());
        let updated_str = note.frontmatter.updated.map(|d| d.to_rfc3339());
        let mtime = note
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                note.id(),
                note.title(),
                note.note_type().to_string(),
                path_str,
                created_str,
                updated_str,
                &note.body,
                mtime,
            ],
        )
        .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note.id(), e)))?;

        let rowid: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![
                rowid,
                note.title(),
                &note.body,
                note.frontmatter.tags.join(" "),
            ],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to insert note {} into FTS: {}",
                note.id(),
                e
            ))
        })?;

        for tag in &note.frontmatter.tags {
            conn.execute(
                "INSERT OR REPLACE INTO tags (note_id, tag) VALUES (?1, ?2)",
                params![note.id(), tag],
            )
            .map_err(|e| QipuError::Other(format!("failed to insert tag {}: {}", tag, e)))?;
        }

        Ok(())
    }

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

    pub(super) fn delete_note_internal(conn: &Connection, note_id: &str) -> Result<()> {
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

    #[allow(clippy::unnecessary_unwrap)]
    pub fn list_notes(
        &self,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        since: Option<chrono::DateTime<Utc>>,
    ) -> Result<Vec<NoteMetadata>> {
        let mut sql = String::from(
            r#"
            SELECT n.id, n.title, n.type, n.path, n.created, n.updated
            FROM notes n
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut has_where = false;

        if type_filter.is_some() {
            sql.push_str(" WHERE n.type = ?");
            params.push(Box::new(type_filter.unwrap().to_string()));
            has_where = true;
        }

        if tag_filter.is_some() {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
                has_where = true;
            }
            sql.push_str(" EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = ?)");
            params.push(Box::new(tag_filter.unwrap().to_string()));
        }

        if since.is_some() {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
                #[allow(unused_assignments)]
                {
                    has_where = true;
                }
            }
            sql.push_str(" n.created >= ?");
            params.push(Box::new(since.unwrap().to_rfc3339()));
        }

        sql.push_str(" ORDER BY n.created DESC, n.id");

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare list query: {}", e)))?;

        let mut results = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut rows = stmt
            .query(param_refs.as_slice())
            .map_err(|e| QipuError::Other(format!("failed to execute list query: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read list results: {}", e)))?
        {
            let id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
            let title: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get title: {}", e)))?;
            let type_str: String = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get type: {}", e)))?;
            let path: String = row
                .get(3)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let created: Option<String> = row
                .get(4)
                .map_err(|e| QipuError::Other(format!("failed to get created: {}", e)))?;
            let updated: Option<String> = row
                .get(5)
                .map_err(|e| QipuError::Other(format!("failed to get updated: {}", e)))?;

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let mut tag_stmt = self
                .conn
                .prepare("SELECT tag FROM tags WHERE note_id = ?1")
                .map_err(|e| QipuError::Other(format!("failed to prepare tag query: {}", e)))?;

            let mut tags = Vec::new();
            let mut tag_rows = tag_stmt
                .query(params![&id])
                .map_err(|e| QipuError::Other(format!("failed to query tags: {}", e)))?;

            while let Some(row) = tag_rows
                .next()
                .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?
            {
                tags.push(
                    row.get(0)
                        .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?,
                );
            }

            results.push(NoteMetadata {
                id,
                title,
                note_type,
                tags,
                path,
                created: created_dt,
                updated: updated_dt,
            });
        }

        Ok(results)
    }
}
