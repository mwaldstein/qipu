//! Full-text search functionality for the database

use crate::lib::error::{QipuError, Result};
use crate::lib::index::types::SearchResult;
use crate::lib::index::weights::{BODY_WEIGHT, TAGS_WEIGHT, TITLE_WEIGHT};
use crate::lib::note::NoteType;
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

impl super::Database {
    /// Perform full-text search using FTS5 with BM25 ranking
    ///
    /// Field weights for BM25 (column weights) are defined in weights.rs:
    /// - Title: TITLE_WEIGHT boost
    /// - Body: BODY_WEIGHT (baseline)
    /// - Tags: TAGS_WEIGHT boost
    pub fn search(
        &self,
        query: &str,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        min_value: Option<u8>,
        equivalent_tags: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Wrap query in double quotes to treat it as a phrase search
        // This prevents FTS5 from interpreting hyphens as column references
        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
        let title_query = format!("title:{}", &fts_query);
        let tags_query = format!("tags:{}", &fts_query);

        let limit_i64 = limit as i64;

        // Build filter conditions for type, tag, and value
        let type_filter_str = type_filter.map(|t| t.to_string());
        let tag_filter_str = tag_filter.map(|t| t.to_string());

        let mut where_clause = String::new();

        if let Some(ref tf) = type_filter_str {
            where_clause.push_str(&format!(" AND n.type = '{}' ", tf));
        }

        // Handle tag filtering with equivalent tags for alias resolution
        if let Some(ref equiv_tags) = equivalent_tags {
            // If multiple equivalent tags exist (alias resolution), use IN clause
            if equiv_tags.len() > 1 {
                let tags_list: Vec<String> =
                    equiv_tags.iter().map(|t| format!("'{}'", t)).collect();
                where_clause.push_str(&format!(
                    " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag IN ({})) ",
                    tags_list.join(", ")
                ));
            } else {
                // Single tag, use simple equality
                where_clause.push_str(&format!(
                    " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = '{}') ",
                    equiv_tags.first().unwrap_or(&tag_filter_str.unwrap_or_default())
                ));
            }
        } else if let Some(ref tg) = tag_filter_str {
            // Fall back to single tag filter if no equivalent tags provided
            where_clause.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = '{}') ",
                tg
            ));
        }

        if let Some(min_val) = min_value {
            where_clause.push_str(&format!(
                " AND (n.value >= {} OR n.value IS NULL) ",
                min_val
            ));
        }

        // Recency boost: decay factor for age in days
        // - Notes updated within 7 days get ~0.1 boost
        // - Notes updated 30+ days ago get minimal boost
        // - Notes updated 90+ days ago get essentially no boost
        // Formula: 0.1 / (1 + age_days / 7)
        // BM25 returns negative scores (closer to 0 is better), so we ADD the boost
        // to make recent notes less negative (higher ranking)
        // Query-specific boosts to ensure proper ranking per spec requirements:
        // - Title matches: +TITLE_WEIGHT (matches BM25 title weight)
        // - Tag matches: +3.0 (higher to ensure tags rank above body even with multiple body occurrences)
        // This ensures "Title matches rank above body matches" and "Exact tag matches rank above plain text"
        // COALESCE handles NULL dates: use updated, then created, then 'now' as fallback
        let sql = format!(
            r#"
            WITH ranked_results AS (
              SELECT
                n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags, n.value, n.created, n.updated,
                bm25(notes_fts, {}, {}, {}) +
                (0.1 / (1.0 + COALESCE((julianday('now') - julianday(COALESCE(n.updated, n.created))), 0.0) / 7.0)) AS rank
              FROM notes_fts
              JOIN notes n ON notes_fts.rowid = n.rowid
              WHERE notes_fts MATCH ?1 {}

              UNION ALL

              SELECT
                n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags, n.value, n.created, n.updated,
                bm25(notes_fts, {}, {}, {}) + {} +
                (0.1 / (1.0 + COALESCE((julianday('now') - julianday(COALESCE(n.updated, n.created))), 0.0) / 7.0)) AS rank
              FROM notes_fts
              JOIN notes n ON notes_fts.rowid = n.rowid
              WHERE notes_fts MATCH ?2 {}

              UNION ALL

              SELECT
                n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags, n.value, n.created, n.updated,
                bm25(notes_fts, {}, {}, {}) + 3.0 +
                (0.1 / (1.0 + COALESCE((julianday('now') - julianday(COALESCE(n.updated, n.created))), 0.0) / 7.0)) AS rank
              FROM notes_fts
              JOIN notes n ON notes_fts.rowid = n.rowid
              WHERE notes_fts MATCH ?3 {}
            )
            SELECT rowid, id, title, path, type, tags, value, created, updated, MAX(rank) AS rank
            FROM ranked_results
            GROUP BY rowid
            ORDER BY rank DESC
            LIMIT ?4
        "#,
            TITLE_WEIGHT,
            BODY_WEIGHT,
            TAGS_WEIGHT,
            where_clause,
            TITLE_WEIGHT,
            BODY_WEIGHT,
            TAGS_WEIGHT,
            TITLE_WEIGHT,
            where_clause,
            TITLE_WEIGHT,
            BODY_WEIGHT,
            TAGS_WEIGHT,
            where_clause
        );

        let params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(fts_query.clone()),
            Box::new(title_query.clone()),
            Box::new(tags_query.clone()),
            Box::new(limit_i64),
        ];

        let mut stmt = self.conn.prepare(&sql).map_err(|e| {
            QipuError::Other(format!(
                "failed to prepare search query for '{}': {}",
                query, e
            ))
        })?;

        let mut results = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

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
            let _rowid: i64 = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get rowid: {}", e)))?;
            let id: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
            let title: String = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get title: {}", e)))?;
            let path: String = row
                .get(3)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let note_type_str: String = row
                .get(4)
                .map_err(|e| QipuError::Other(format!("failed to get type: {}", e)))?;
            let tags_str: String = row
                .get(5)
                .map_err(|e| QipuError::Other(format!("failed to get tags: {}", e)))?;
            let value: Option<i64> = row
                .get(6)
                .map_err(|e| QipuError::Other(format!("failed to get value: {}", e)))?;
            let created: Option<String> = row
                .get(7)
                .map_err(|e| QipuError::Other(format!("failed to get created: {}", e)))?;
            let updated: Option<String> = row
                .get(8)
                .map_err(|e| QipuError::Other(format!("failed to get updated: {}", e)))?;
            let rank: f64 = row
                .get(9)
                .map_err(|e| QipuError::Other(format!("failed to get rank: {}", e)))?;

            let note_type =
                NoteType::from_str(&note_type_str).map_err(convert_qipu_error_to_sqlite)?;
            let tags = parse_tags(&tags_str);
            let value_opt = value.and_then(|v| u8::try_from(v).ok());
            let created_opt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_opt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            results.push(SearchResult {
                id,
                title,
                note_type,
                tags,
                path,
                match_context: None,
                relevance: rank,
                via: None,
                value: value_opt,
                created: created_opt,
                updated: updated_opt,
            });
        }

        Ok(results)
    }
}
