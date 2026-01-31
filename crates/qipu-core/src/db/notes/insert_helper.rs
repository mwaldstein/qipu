use crate::error::{QipuError, Result};
use crate::note::Note;
use rusqlite::params;

/// Options for note insertion behavior
pub enum InsertOptions {
    /// Standard insertion with full FTS indexing, no default value
    Standard,
    /// Internal insertion with full FTS indexing, default value of 50
    Internal,
    /// Basic index level (metadata only, empty body in FTS), default value of 50
    Basic,
}

impl InsertOptions {
    fn is_basic(&self) -> bool {
        matches!(self, InsertOptions::Basic)
    }

    fn should_set_default_value(&self) -> bool {
        matches!(self, InsertOptions::Internal | InsertOptions::Basic)
    }

    fn should_index_body(&self) -> bool {
        !matches!(self, InsertOptions::Basic)
    }
}

/// Insert note into database using provided connection
pub fn insert_note_with_options(
    conn: &rusqlite::Connection,
    note: &Note,
    options: InsertOptions,
) -> Result<()> {
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
    let note_id = note.id().to_string();
    let title = note.title().to_string();
    let note_type = note.note_type().to_string();
    let value = if options.should_set_default_value() {
        note.frontmatter.value.map(|v| v as i32).or(Some(50))
    } else {
        note.frontmatter.value.map(|v| v as i32)
    };

    if options.is_basic() {
        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json, index_level)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                &note_id,
                &title,
                &note_type,
                path_str,
                &created_str,
                &updated_str,
                &note.body,
                mtime,
                &value,
                &compacts_json,
                note.frontmatter.author.as_ref(),
                verified_int,
                note.frontmatter.source.as_ref(),
                &sources_json,
                note.frontmatter.generated_by.as_ref(),
                note.frontmatter.prompt_hash.as_ref(),
                &custom_json,
                1,
            ],
        )
        .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note_id, e)))?;
    } else {
        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                &note_id,
                &title,
                &note_type,
                path_str,
                &created_str,
                &updated_str,
                &note.body,
                mtime,
                &value,
                &compacts_json,
                note.frontmatter.author.as_ref(),
                verified_int,
                note.frontmatter.source.as_ref(),
                &sources_json,
                note.frontmatter.generated_by.as_ref(),
                note.frontmatter.prompt_hash.as_ref(),
                &custom_json,
            ],
        )
        .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note_id, e)))?;
    }

    let rowid: i64 = conn.last_insert_rowid();

    // Insert into FTS5
    let tags_str = note.frontmatter.tags.join(" ");
    let fts_body = if options.should_index_body() {
        &note.body
    } else {
        ""
    };

    conn.execute(
        "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
        params![rowid, &title, fts_body, &tags_str],
    )
    .map_err(|e| QipuError::Other(format!("failed to insert note {} into FTS: {}", note_id, e)))?;

    // Insert tags
    for tag in &note.frontmatter.tags {
        conn.execute(
            "INSERT OR REPLACE INTO tags (note_id, tag) VALUES (?1, ?2)",
            params![&note_id, tag],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to insert tag '{}' for note {}: {}",
                tag, note_id, e
            ))
        })?;
    }

    Ok(())
}
