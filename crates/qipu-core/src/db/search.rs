//! Full-text search functionality for the database

use crate::config::SearchConfig;
use crate::error::{QipuError, Result};
use crate::index::types::SearchResult;
use crate::index::weights::{BODY_WEIGHT, TAGS_WEIGHT, TITLE_WEIGHT};
use crate::note::NoteType;
use rusqlite::Row;
use std::str::FromStr;

fn convert_qipu_error_to_sqlite(e: QipuError) -> rusqlite::Error {
    match e {
        QipuError::Other(msg) => rusqlite::Error::ToSqlConversionFailure(Box::from(msg)),
        _ => rusqlite::Error::ToSqlConversionFailure(Box::from(format!("{:?}", e))),
    }
}

/// Parse tags from a space-separated string
fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

struct SearchQuery {
    sql: String,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

struct SearchFilterSql {
    clause: String,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

fn build_search_filters(
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    min_value: Option<u8>,
    equivalent_tags: Option<&[String]>,
) -> SearchFilterSql {
    let mut clause = String::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut next_param = 5;

    if let Some(note_type) = type_filter {
        clause.push_str(&format!(" AND n.type = ?{} ", next_param));
        params.push(Box::new(note_type.to_string()));
        next_param += 1;
    }

    if let Some(tags) = equivalent_tags.filter(|tags| !tags.is_empty()) {
        let placeholders: Vec<String> = tags
            .iter()
            .map(|tag| {
                let placeholder = format!("?{}", next_param);
                next_param += 1;
                params.push(Box::new(tag.clone()));
                placeholder
            })
            .collect();
        clause.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag IN ({})) ",
            placeholders.join(", ")
        ));
    } else if let Some(tag) = tag_filter {
        clause.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = ?{}) ",
            next_param
        ));
        params.push(Box::new(tag.to_string()));
        next_param += 1;
    }

    if let Some(min_val) = min_value {
        clause.push_str(&format!(" AND COALESCE(n.value, 50) >= ?{} ", next_param));
        params.push(Box::new(i64::from(min_val)));
    }

    SearchFilterSql { clause, params }
}

fn recency_formula(search_config: &SearchConfig) -> String {
    format!(
        "({} / (1.0 + COALESCE((julianday('now') - julianday(COALESCE(n.updated, n.created))), 0.0) / {}))",
        search_config.recency_boost_numerator,
        search_config.recency_decay_days
    )
}

fn build_search_sql(where_clause: &str, search_config: &SearchConfig) -> String {
    let recency_formula = recency_formula(search_config);
    let weighted_rank = format!(
        "bm25(notes_fts, {}, {}, {}) + {} AS rank",
        TITLE_WEIGHT, BODY_WEIGHT, TAGS_WEIGHT, recency_formula
    );

    format!(
        r#"
        WITH ranked_results AS (
          SELECT n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags,
                 n.value, n.created, n.updated, {weighted_rank}
          FROM notes_fts
          JOIN notes n ON notes_fts.rowid = n.rowid
          WHERE notes_fts MATCH ?1 {where_clause}

          UNION ALL

          SELECT n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags,
                 n.value, n.created, n.updated, {weighted_rank}
          FROM notes_fts
          JOIN notes n ON notes_fts.rowid = n.rowid
          WHERE notes_fts MATCH ?2 {where_clause}

          UNION ALL

          SELECT n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags,
                 n.value, n.created, n.updated, {weighted_rank}
          FROM notes_fts
          JOIN notes n ON notes_fts.rowid = n.rowid
          WHERE notes_fts MATCH ?3 {where_clause}
        )
        SELECT rowid, id, title, path, type, tags, value, created, updated, MAX(rank) AS rank
        FROM ranked_results
        GROUP BY rowid
        ORDER BY rank DESC
        LIMIT ?4
        "#
    )
}

fn build_search_query(
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    min_value: Option<u8>,
    equivalent_tags: Option<&[String]>,
    limit: usize,
    search_config: &SearchConfig,
) -> SearchQuery {
    let fts_query = query.replace('-', " ").replace('"', "\"\"");
    let title_query = format!("title:{}", &fts_query);
    let tags_query = format!("tags:{}", &fts_query);
    let filters = build_search_filters(type_filter, tag_filter, min_value, equivalent_tags);

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(fts_query),
        Box::new(title_query),
        Box::new(tags_query),
        Box::new(limit as i64),
    ];
    params.extend(filters.params);

    SearchQuery {
        sql: build_search_sql(&filters.clause, search_config),
        params,
    }
}

fn search_result_from_row(row: &Row<'_>) -> rusqlite::Result<SearchResult> {
    let note_type_str: String = row.get(4)?;
    let note_type = NoteType::from_str(&note_type_str).map_err(convert_qipu_error_to_sqlite)?;
    let tags_str: String = row.get(5)?;
    let value: Option<i64> = row.get(6)?;
    let created: Option<String> = row.get(7)?;
    let updated: Option<String> = row.get(8)?;

    Ok(SearchResult {
        id: row.get(1)?,
        title: row.get(2)?,
        note_type,
        tags: parse_tags(&tags_str),
        path: row.get(3)?,
        match_context: None,
        relevance: row.get(9)?,
        via: None,
        value: value.and_then(|v| u8::try_from(v).ok()),
        created: parse_rfc3339_utc(created),
        updated: parse_rfc3339_utc(updated),
    })
}

fn parse_rfc3339_utc(value: Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    value
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

impl super::Database {
    /// Perform full-text search using FTS5 with BM25 ranking
    ///
    /// Field weights for BM25 (column weights) are defined in weights.rs:
    /// - Title: TITLE_WEIGHT boost
    /// - Body: BODY_WEIGHT (baseline)
    /// - Tags: TAGS_WEIGHT boost
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        query: &str,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        min_value: Option<u8>,
        equivalent_tags: Option<&[String]>,
        limit: usize,
        search_config: &SearchConfig,
    ) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let search_query = build_search_query(
            query,
            type_filter,
            tag_filter,
            min_value,
            equivalent_tags,
            limit,
            search_config,
        );
        let mut stmt = self.conn.prepare(&search_query.sql).map_err(|e| {
            QipuError::Other(format!(
                "failed to prepare search query for '{}': {}",
                query, e
            ))
        })?;

        let mut results = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            search_query.params.iter().map(|p| p.as_ref()).collect();

        let mut rows = stmt.query(param_refs.as_slice()).map_err(|e| {
            QipuError::Other(format!(
                "failed to execute search query for '{}': {}",
                query, e
            ))
        })?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read search results: {}", e)))?
        {
            results.push(search_result_from_row(row).map_err(|e| {
                QipuError::Other(format!("failed to read search result row: {}", e))
            })?);
        }

        Ok(results)
    }
}
