use crate::error::{QipuError, Result};
use crate::note::{Note, NoteType};
use rusqlite::{params, Connection};

impl super::super::Database {
    pub fn insert_note(&self, note: &Note) -> Result<()> {
        let created_str = note.frontmatter.created.map(|dt| dt.to_rfc3339());
        let updated_str = note.frontmatter.updated.map(|dt| dt.to_rfc3339());
        let mtime = note
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let tags_str = note.frontmatter.tags.join(" ");
        let compacts_json = note.frontmatter.to_compacts_json();
        let sources_json = note.frontmatter.to_sources_json();
        let verified_int = note.frontmatter.verified.map(|b| if b { 1 } else { 0 });
        let custom_json = note.frontmatter.to_custom_json();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    note.id(),
                    note.frontmatter.title,
                    note.frontmatter.note_type.as_ref().unwrap_or(&NoteType::from(NoteType::FLEETING)).to_string(),
                    note.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                    created_str,
                    updated_str,
                    &note.body,
                    mtime,
                    note.frontmatter.value,
                    compacts_json,
                    note.frontmatter.author.as_ref(),
                    verified_int,
                    note.frontmatter.source.as_ref(),
                    sources_json,
                    note.frontmatter.generated_by.as_ref(),
                    note.frontmatter.prompt_hash.as_ref(),
                    custom_json,
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

    pub(crate) fn insert_note_internal(conn: &Connection, note: &Note) -> Result<()> {
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
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let compacts_json = note.frontmatter.to_compacts_json();
        let sources_json = note.frontmatter.to_sources_json();
        let verified_int = note.frontmatter.verified.map(|b| if b { 1 } else { 0 });
        let custom_json = note.frontmatter.to_custom_json();

        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                note.id(),
                note.title(),
                note.note_type().to_string(),
                path_str,
                created_str,
                updated_str,
                &note.body,
                mtime,
                note.frontmatter.value.or(Some(50)),
                compacts_json,
                note.frontmatter.author.as_ref(),
                verified_int,
                note.frontmatter.source.as_ref(),
                sources_json,
                note.frontmatter.generated_by.as_ref(),
                note.frontmatter.prompt_hash.as_ref(),
                custom_json,
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
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to insert tag '{}' for note {}: {}",
                    tag,
                    note.id(),
                    e
                ))
            })?;
        }

        Ok(())
    }
}
