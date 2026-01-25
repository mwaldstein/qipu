use crate::lib::error::{QipuError, Result};
use crate::lib::note::{NoteFrontmatter, NoteType, TypedLink};
use chrono::Utc;
use rusqlite::params;
use std::str::FromStr;

fn convert_qipu_error_to_sqlite(e: QipuError) -> rusqlite::Error {
    match e {
        QipuError::Other(msg) => rusqlite::Error::ToSqlConversionFailure(Box::from(msg)),
        _ => rusqlite::Error::ToSqlConversionFailure(Box::from(format!("{:?}", e))),
    }
}

pub fn parse_note_type_sqlite(type_str: &str) -> std::result::Result<NoteType, rusqlite::Error> {
    NoteType::from_str(type_str).map_err(convert_qipu_error_to_sqlite)
}

pub fn parse_note_type(type_str: &str) -> Result<NoteType> {
    NoteType::from_str(type_str)
        .map_err(|_| QipuError::Other(format!("invalid note type: {}", type_str)))
}

pub fn parse_datetime(opt_string: Option<String>) -> Option<chrono::DateTime<Utc>> {
    opt_string
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub fn parse_value(opt_value: Option<i64>) -> Option<u8> {
    opt_value.and_then(|v| u8::try_from(v).ok())
}

pub fn parse_verified(opt_verified: Option<i64>) -> Option<bool> {
    opt_verified.map(|v| v != 0)
}

pub fn load_tags(conn: &rusqlite::Connection, note_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT tag FROM tags WHERE note_id = ?1")
        .map_err(|e| QipuError::Other(format!("failed to prepare tag query: {}", e)))?;

    let mut tags = Vec::new();
    let mut tag_rows = stmt
        .query(params![note_id])
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

    Ok(tags)
}

pub fn load_links(conn: &rusqlite::Connection, note_id: &str) -> Result<Vec<TypedLink>> {
    let mut stmt = conn
        .prepare("SELECT target_id, link_type, inline FROM edges WHERE source_id = ?1")
        .map_err(|e| QipuError::Other(format!("failed to prepare edge query: {}", e)))?;

    let mut links = Vec::new();
    let mut edge_rows = stmt
        .query(params![note_id])
        .map_err(|e| QipuError::Other(format!("failed to query edges: {}", e)))?;

    while let Some(row) = edge_rows
        .next()
        .map_err(|e| QipuError::Other(format!("failed to read edge: {}", e)))?
    {
        let target_id: String = row.get(0)?;
        let link_type_str: String = row.get(1)?;
        let inline: i64 = row.get(2)?;

        if inline == 0 {
            links.push(TypedLink {
                id: target_id,
                link_type: link_type_str.into(),
            });
        }
    }

    Ok(links)
}

pub fn load_compacts(json_str: &str) -> Vec<String> {
    serde_json::from_str(json_str).unwrap_or_default()
}

pub fn load_sources(json_str: &str) -> Vec<crate::lib::note::Source> {
    serde_json::from_str(json_str).unwrap_or_default()
}

pub fn load_custom(json_str: &str) -> std::collections::HashMap<String, serde_yaml::Value> {
    serde_json::from_str(json_str).unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub fn build_frontmatter(
    id: String,
    title: String,
    note_type: NoteType,
    created: Option<chrono::DateTime<Utc>>,
    updated: Option<chrono::DateTime<Utc>>,
    tags: Vec<String>,
    sources: Vec<crate::lib::note::Source>,
    links: Vec<TypedLink>,
    compacts: Vec<String>,
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
    value: Option<u8>,
    custom: std::collections::HashMap<String, serde_yaml::Value>,
) -> NoteFrontmatter {
    NoteFrontmatter {
        id: id.clone(),
        title,
        note_type: Some(note_type),
        created,
        updated,
        tags,
        sources,
        links,
        summary: None,
        compacts,
        source,
        author,
        generated_by,
        prompt_hash,
        verified,
        value,
        custom,
    }
}
