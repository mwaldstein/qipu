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

struct NoteInsertData<'a> {
    path: &'a str,
    created: Option<String>,
    updated: Option<String>,
    mtime: i64,
    compacts: String,
    sources: String,
    verified: Option<i32>,
    custom: String,
    id: String,
    title: String,
    note_type: String,
    value: Option<i32>,
}

impl<'a> NoteInsertData<'a> {
    fn from_note(note: &'a Note, options: &InsertOptions) -> Result<Self> {
        let path = note
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or_else(|| QipuError::Other(format!("invalid path for note {}", note.id())))?;
        let value = if options.should_set_default_value() {
            note.frontmatter.value.map(|v| v as i32).or(Some(50))
        } else {
            note.frontmatter.value.map(|v| v as i32)
        };

        Ok(Self {
            path,
            created: note.frontmatter.created.map(|d| d.to_rfc3339()),
            updated: note.frontmatter.updated.map(|d| d.to_rfc3339()),
            mtime: note_mtime(note),
            compacts: note.frontmatter.to_compacts_json(),
            sources: note.frontmatter.to_sources_json(),
            verified: note.frontmatter.verified.map(|b| if b { 1 } else { 0 }),
            custom: note.frontmatter.to_custom_json(),
            id: note.id().to_string(),
            title: note.title().to_string(),
            note_type: note.note_type().to_string(),
            value,
        })
    }
}

fn note_mtime(note: &Note) -> i64 {
    note.path
        .as_ref()
        .and_then(|p| std::fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

/// Insert note into database using provided connection
pub fn insert_note_with_options(
    conn: &rusqlite::Connection,
    note: &Note,
    options: InsertOptions,
) -> Result<()> {
    let data = NoteInsertData::from_note(note, &options)?;

    if options.is_basic() {
        insert_basic_note(conn, note, &data)?;
    } else {
        insert_standard_note(conn, note, &data)?;
    }

    let rowid: i64 = conn.last_insert_rowid();
    insert_note_fts(conn, note, &data, rowid, &options)?;
    insert_note_tags(conn, note, &data.id)?;

    Ok(())
}

fn insert_basic_note(
    conn: &rusqlite::Connection,
    note: &Note,
    data: &NoteInsertData<'_>,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json, index_level)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            &data.id,
            &data.title,
            &data.note_type,
            data.path,
            &data.created,
            &data.updated,
            &note.body,
            data.mtime,
            &data.value,
            &data.compacts,
            note.frontmatter.author.as_ref(),
            data.verified,
            note.frontmatter.source.as_ref(),
            &data.sources,
            note.frontmatter.generated_by.as_ref(),
            note.frontmatter.prompt_hash.as_ref(),
            &data.custom,
            1,
        ],
    )
    .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", data.id, e)))?;
    Ok(())
}

fn insert_standard_note(
    conn: &rusqlite::Connection,
    note: &Note,
    data: &NoteInsertData<'_>,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            &data.id,
            &data.title,
            &data.note_type,
            data.path,
            &data.created,
            &data.updated,
            &note.body,
            data.mtime,
            &data.value,
            &data.compacts,
            note.frontmatter.author.as_ref(),
            data.verified,
            note.frontmatter.source.as_ref(),
            &data.sources,
            note.frontmatter.generated_by.as_ref(),
            note.frontmatter.prompt_hash.as_ref(),
            &data.custom,
        ],
    )
    .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", data.id, e)))?;
    Ok(())
}

fn insert_note_fts(
    conn: &rusqlite::Connection,
    note: &Note,
    data: &NoteInsertData<'_>,
    rowid: i64,
    options: &InsertOptions,
) -> Result<()> {
    let tags_str = note.frontmatter.tags.join(" ");
    let fts_body = if options.should_index_body() {
        &note.body
    } else {
        ""
    };

    conn.execute(
        "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
        params![rowid, &data.title, fts_body, &tags_str],
    )
    .map_err(|e| QipuError::Other(format!("failed to insert note {} into FTS: {}", data.id, e)))?;
    Ok(())
}

fn insert_note_tags(conn: &rusqlite::Connection, note: &Note, note_id: &str) -> Result<()> {
    for tag in &note.frontmatter.tags {
        conn.execute(
            "INSERT OR REPLACE INTO tags (note_id, tag) VALUES (?1, ?2)",
            params![note_id, tag],
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
