use crate::error::{QipuError, Result};
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
        id: row
            .get(0)
            .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?,
        title: row
            .get(1)
            .map_err(|e| QipuError::Other(format!("failed to get title: {}", e)))?,
        type_str: row
            .get(2)
            .map_err(|e| QipuError::Other(format!("failed to get type: {}", e)))?,
        path: row
            .get(3)
            .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?,
        created: row
            .get(4)
            .map_err(|e| QipuError::Other(format!("failed to get created: {}", e)))?,
        updated: row
            .get(5)
            .map_err(|e| QipuError::Other(format!("failed to get updated: {}", e)))?,
        body: row
            .get(6)
            .map_err(|e| QipuError::Other(format!("failed to get body: {}", e)))?,
        value: row
            .get(7)
            .map_err(|e| QipuError::Other(format!("failed to get value: {}", e)))?,
        compacts_json: row
            .get(8)
            .map_err(|e| QipuError::Other(format!("failed to get compacts: {}", e)))?,
        author: row
            .get(9)
            .map_err(|e| QipuError::Other(format!("failed to get author: {}", e)))?,
        verified: row
            .get(10)
            .map_err(|e| QipuError::Other(format!("failed to get verified: {}", e)))?,
        source: row
            .get(11)
            .map_err(|e| QipuError::Other(format!("failed to get source: {}", e)))?,
        sources_json: row
            .get(12)
            .map_err(|e| QipuError::Other(format!("failed to get sources: {}", e)))?,
        generated_by: row
            .get(13)
            .map_err(|e| QipuError::Other(format!("failed to get generated_by: {}", e)))?,
        prompt_hash: row
            .get(14)
            .map_err(|e| QipuError::Other(format!("failed to get prompt_hash: {}", e)))?,
        custom_json: row
            .get(15)
            .map_err(|e| QipuError::Other(format!("failed to get custom_json: {}", e)))?,
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

    #[allow(clippy::unnecessary_unwrap)]
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

        let note_opt = stmt.query_row(params![note_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let path: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;
            let body: String = row.get(6)?;
            let value: Option<i64> = row.get(7)?;
            let compacts_json: String = row.get(8)?;
            let author: Option<String> = row.get(9)?;
            let verified: Option<i64> = row.get(10)?;
            let source: Option<String> = row.get(11)?;
            let sources_json: String = row.get(12)?;
            let generated_by: Option<String> = row.get(13)?;
            let prompt_hash: Option<String> = row.get(14)?;
            let custom_json: String = row.get(15)?;

            let note_type = parse_note_type_sqlite(&type_str)?;
            let created_dt = parse_datetime(created);
            let updated_dt = parse_datetime(updated);
            let value_opt = parse_value(value);
            let verified_opt = parse_verified(verified);

            Ok((
                id,
                title,
                note_type,
                path,
                created_dt,
                updated_dt,
                body,
                value_opt,
                compacts_json,
                author,
                verified_opt,
                source,
                sources_json,
                generated_by,
                prompt_hash,
                custom_json,
            ))
        });

        match note_opt {
            Ok(raw) => self.build_note_from_query_result(raw),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!("failed to query note: {}", e))),
        }
    }

    fn build_note_from_query_result(
        &self,
        (
            id,
            title,
            note_type,
            path,
            created,
            updated,
            body,
            value,
            compacts_json,
            author,
            verified,
            source,
            sources_json,
            generated_by,
            prompt_hash,
            custom_json,
        ): (
            String,
            String,
            NoteType,
            String,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            String,
            Option<u8>,
            String,
            Option<String>,
            Option<bool>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            String,
        ),
    ) -> Result<Option<Note>> {
        let tags = load_tags(&self.conn, &id)?;
        let links = load_links(&self.conn, &id)?;
        let compacts = load_compacts(&compacts_json);
        let sources = load_sources(&sources_json);
        let custom = load_custom(&custom_json);

        let frontmatter = build_frontmatter(
            id.clone(),
            title,
            note_type,
            created,
            updated,
            tags,
            sources,
            links,
            compacts,
            source,
            author,
            generated_by,
            prompt_hash,
            verified,
            value,
            custom,
        );

        Ok(Some(Note {
            frontmatter,
            body,
            path: Some(PathBuf::from(path)),
        }))
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
