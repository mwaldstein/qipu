use crate::error::{QipuError, Result};
use crate::extract;
use crate::index::types::NoteMetadata;
use crate::note::{Note, NoteType};
use chrono::Utc;
use rusqlite::params;
use std::path::PathBuf;

use super::helpers::{
    build_frontmatter, load_compacts, load_custom, load_links, load_sources, load_tags,
    parse_datetime, parse_note_type, parse_note_type_sqlite, parse_value, parse_verified,
};

struct ExtractedNoteRow {
    id: String,
    title: String,
    type_str: String,
    path: String,
    created: Option<String>,
    updated: Option<String>,
    body: String,
    value: Option<i64>,
    compacts_json: String,
    author: Option<String>,
    verified: Option<i64>,
    source: Option<String>,
    sources_json: String,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    custom_json: String,
}

fn extract_note_row(row: &rusqlite::Row) -> Result<ExtractedNoteRow> {
    Ok(ExtractedNoteRow {
        id: extract!(row, 0, "id")?,
        title: extract!(row, 1, "title")?,
        type_str: extract!(row, 2, "type")?,
        path: extract!(row, 3, "path")?,
        created: extract!(row, 4, "created")?,
        updated: extract!(row, 5, "updated")?,
        body: extract!(row, 6, "body")?,
        value: extract!(row, 7, "value")?,
        compacts_json: extract!(row, 8, "compacts")?,
        author: extract!(row, 9, "author")?,
        verified: extract!(row, 10, "verified")?,
        source: extract!(row, 11, "source")?,
        sources_json: extract!(row, 12, "sources")?,
        generated_by: extract!(row, 13, "generated_by")?,
        prompt_hash: extract!(row, 14, "prompt_hash")?,
        custom_json: extract!(row, 15, "custom_json")?,
    })
}

impl super::super::Database {
    pub fn get_max_mtime(&self) -> Result<Option<i64>> {
        self.conn
            .query_row("SELECT MAX(mtime) FROM notes", [], |row| row.get(0))
            .map_err(|e| QipuError::Other(format!("failed to query max mtime: {}", e)))
    }
    pub fn get_note_metadata(&self, note_id: &str) -> Result<Option<NoteMetadata>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, type, path, created, updated, value FROM notes WHERE id = ?1",
            )
            .map_err(|e| QipuError::Other(format!("failed to prepare query: {}", e)))?;

        let note_opt = stmt.query_row(params![note_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let path: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;
            let value: Option<i64> = row.get(6)?;

            let note_type = parse_note_type_sqlite(&type_str)?;
            let created_dt = parse_datetime(created);
            let updated_dt = parse_datetime(updated);
            let value_opt = parse_value(value);

            Ok((
                id, title, note_type, path, created_dt, updated_dt, value_opt,
            ))
        });

        match note_opt {
            Ok((id, title, note_type, path, created, updated, value)) => {
                let tags = load_tags(&self.conn, &id)?;

                Ok(Some(NoteMetadata {
                    id,
                    title,
                    note_type,
                    tags,
                    path,
                    created,
                    updated,
                    value,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!(
                "failed to query note metadata: {}",
                e
            ))),
        }
    }

    pub fn list_notes(
        &self,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        since: Option<chrono::DateTime<Utc>>,
    ) -> Result<Vec<NoteMetadata>> {
        let mut sql = String::from(
            r#"
            SELECT n.id, n.title, n.type, n.path, n.created, n.updated, n.value
            FROM notes n
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut has_where = false;

        if let Some(filter) = type_filter {
            sql.push_str(" WHERE n.type = ?");
            params.push(Box::new(filter.to_string()));
            has_where = true;
        }

        if let Some(filter) = tag_filter {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
                has_where = true;
            }
            sql.push_str(" EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = ?)");
            params.push(Box::new(filter.to_string()));
        }

        if let Some(dt) = since {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
            }
            sql.push_str(" n.created >= ?");
            params.push(Box::new(dt.to_rfc3339()));
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
            let value: Option<i64> = row
                .get(6)
                .map_err(|e| QipuError::Other(format!("failed to get value: {}", e)))?;

            let note_type = parse_note_type(&type_str)?;
            let created_dt = parse_datetime(created);
            let updated_dt = parse_datetime(updated);
            let value_opt = parse_value(value);

            let tags = load_tags(&self.conn, &id)?;

            results.push(NoteMetadata {
                id,
                title,
                note_type,
                tags,
                path,
                created: created_dt,
                updated: updated_dt,
                value: value_opt,
            });
        }

        Ok(results)
    }

    pub fn get_note(&self, note_id: &str) -> Result<Option<Note>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, type, path, created, updated, body, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json FROM notes WHERE id = ?1",
            )
            .map_err(|e| QipuError::Other(format!("failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to execute query: {}", e)))?;

        let row_opt = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read note row: {}", e)))?;

        match row_opt {
            Some(row) => {
                let note = self.build_note_from_row(row)?;
                Ok(Some(note))
            }
            None => Ok(None),
        }
    }

    pub fn list_note_ids(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM notes ORDER BY id")
            .map_err(|e| QipuError::Other(format!("failed to prepare note IDs query: {}", e)))?;

        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| QipuError::Other(format!("failed to query note IDs: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read note ID rows: {}", e)))?;

        Ok(ids)
    }

    pub fn list_notes_full(&self) -> Result<Vec<Note>> {
        let sql = r#"
            SELECT id, title, type, path, created, updated, body, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json
            FROM notes
            ORDER BY created DESC, id
        "#;

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare list query: {}", e)))?;

        let mut results = Vec::new();

        let mut rows = stmt
            .query([])
            .map_err(|e| QipuError::Other(format!("failed to execute list query: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read list results: {}", e)))?
        {
            let note = self.build_note_from_row(row)?;
            results.push(note);
        }

        Ok(results)
    }

    fn build_note_from_row(&self, row: &rusqlite::Row) -> Result<Note> {
        let raw = extract_note_row(row)?;
        self.build_note_from_raw(raw)
    }

    fn build_note_from_raw(&self, raw: ExtractedNoteRow) -> Result<Note> {
        let note_type = parse_note_type_sqlite(&raw.type_str)?;
        let created_dt = parse_datetime(raw.created);
        let updated_dt = parse_datetime(raw.updated);
        let value_opt = parse_value(raw.value);
        let verified_opt = parse_verified(raw.verified);

        let tags = load_tags(&self.conn, &raw.id)?;
        let links = load_links(&self.conn, &raw.id)?;
        let compacts = load_compacts(&raw.compacts_json);
        let sources = load_sources(&raw.sources_json);
        let custom = load_custom(&raw.custom_json);

        let frontmatter = build_frontmatter(
            raw.id.clone(),
            raw.title,
            note_type,
            created_dt,
            updated_dt,
            tags,
            sources,
            links,
            compacts,
            raw.source,
            raw.author,
            raw.generated_by,
            raw.prompt_hash,
            verified_opt,
            value_opt,
            custom,
        );

        Ok(Note {
            frontmatter,
            body: raw.body,
            path: Some(PathBuf::from(raw.path)),
        })
    }

    pub fn get_tag_frequencies(&self) -> Result<Vec<(String, i64)>> {
        let sql = r#"
            SELECT tag, COUNT(*) as count
            FROM tags
            GROUP BY tag
            ORDER BY count DESC, tag
        "#;

        let mut stmt = self.conn.prepare(sql).map_err(|e| {
            QipuError::Other(format!("failed to prepare tag frequency query: {}", e))
        })?;

        let mut results = Vec::new();

        let mut rows = stmt.query([]).map_err(|e| {
            QipuError::Other(format!("failed to execute tag frequency query: {}", e))
        })?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read tag frequency results: {}", e)))?
        {
            let tag: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?;
            let count: i64 = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to read count: {}", e)))?;
            results.push((tag, count));
        }

        Ok(results)
    }
}
