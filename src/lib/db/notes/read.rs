use crate::lib::error::{QipuError, Result};
use crate::lib::index::types::NoteMetadata;
use crate::lib::note::{Note, NoteFrontmatter, NoteType};
use chrono::Utc;
use rusqlite::params;
use std::path::PathBuf;
use std::str::FromStr;

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

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let value_opt = value.and_then(|v| u8::try_from(v).ok());

            Ok((
                id, title, note_type, path, created_dt, updated_dt, value_opt,
            ))
        });

        match note_opt {
            Ok((id, title, note_type, path, created, updated, value)) => {
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

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let value_opt = value.and_then(|v| u8::try_from(v).ok());

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
                value: value_opt,
            });
        }

        Ok(results)
    }

    pub fn get_note(&self, note_id: &str) -> Result<Option<Note>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, type, path, created, updated, body, value FROM notes WHERE id = ?1",
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

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let value_opt = value.and_then(|v| u8::try_from(v).ok());

            Ok((
                id, title, note_type, path, created_dt, updated_dt, body, value_opt,
            ))
        });

        match note_opt {
            Ok((id, title, note_type, path, created, updated, body, value)) => {
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

                let mut edge_stmt = self
                    .conn
                    .prepare("SELECT target_id, link_type, inline FROM edges WHERE source_id = ?1")
                    .map_err(|e| {
                        QipuError::Other(format!("failed to prepare edge query: {}", e))
                    })?;

                let mut links = Vec::new();
                let mut edge_rows = edge_stmt
                    .query(params![&id])
                    .map_err(|e| QipuError::Other(format!("failed to query edges: {}", e)))?;

                while let Some(row) = edge_rows
                    .next()
                    .map_err(|e| QipuError::Other(format!("failed to read edge: {}", e)))?
                {
                    let target_id: String = row.get(0)?;
                    let link_type_str: String = row.get(1)?;

                    let link_type = crate::lib::note::LinkType::from(link_type_str);
                    links.push(crate::lib::note::TypedLink {
                        id: target_id,
                        link_type,
                    });
                }

                let frontmatter = NoteFrontmatter {
                    id: id.clone(),
                    title,
                    note_type: Some(note_type),
                    created,
                    updated,
                    tags,
                    sources: Vec::new(),
                    links,
                    summary: None,
                    compacts: Vec::new(),
                    source: None,
                    author: None,
                    generated_by: None,
                    prompt_hash: None,
                    verified: None,
                    value,
                };

                Ok(Some(Note {
                    frontmatter,
                    body,
                    path: Some(PathBuf::from(path)),
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!("failed to query note: {}", e))),
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
            SELECT id, title, type, path, created, updated, body, value
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
            let body: String = row
                .get(6)
                .map_err(|e| QipuError::Other(format!("failed to get body: {}", e)))?;
            let value: Option<i64> = row
                .get(7)
                .map_err(|e| QipuError::Other(format!("failed to get value: {}", e)))?;

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let value_opt = value.and_then(|v| u8::try_from(v).ok());

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

            let mut edge_stmt = self
                .conn
                .prepare("SELECT target_id, link_type FROM edges WHERE source_id = ?1")
                .map_err(|e| QipuError::Other(format!("failed to prepare edge query: {}", e)))?;

            let mut links = Vec::new();
            let mut edge_rows = edge_stmt
                .query(params![&id])
                .map_err(|e| QipuError::Other(format!("failed to query edges: {}", e)))?;

            while let Some(row) = edge_rows
                .next()
                .map_err(|e| QipuError::Other(format!("failed to read edge: {}", e)))?
            {
                let target_id: String = row.get(0)?;
                let link_type_str: String = row.get(1)?;

                let link_type = crate::lib::note::LinkType::from(link_type_str);
                links.push(crate::lib::note::TypedLink {
                    id: target_id,
                    link_type,
                });
            }

            let frontmatter = NoteFrontmatter {
                id: id.clone(),
                title,
                note_type: Some(note_type),
                created: created_dt,
                updated: updated_dt,
                tags,
                sources: Vec::new(),
                links,
                summary: None,
                compacts: Vec::new(),
                source: None,
                author: None,
                generated_by: None,
                prompt_hash: None,
                verified: None,
                value: value_opt,
            };

            results.push(Note {
                frontmatter,
                body,
                path: Some(PathBuf::from(path)),
            });
        }

        Ok(results)
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
