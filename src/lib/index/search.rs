use super::types::{Index, SearchResult};
use crate::lib::error::Result;
use crate::lib::logging;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use crate::lib::text::{calculate_bm25, tokenize};
use chrono::Utc;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Ripgrep JSON output variants
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RipgrepMatch {
    Begin {
        path: String,
    },
    End {
        path: String,
    },
    Match {
        path: String,
        lines: String,
        #[allow(dead_code)]
        line_number: u64,
        #[allow(dead_code)]
        absolute_offset: u64,
    },
}

/// Calculate recency boost based on how recently a note was updated.
/// Returns a small boost (0.0 to 0.5) that decays exponentially over time.
/// - Notes updated within 7 days: full boost (0.5)
/// - Notes updated within 30 days: moderate boost (0.25)
/// - Notes updated within 90 days: small boost (0.1)
/// - Notes updated over 90 days ago or without timestamp: no boost (0.0)
fn calculate_recency_boost(updated: Option<chrono::DateTime<Utc>>) -> f64 {
    let Some(updated) = updated else {
        return 0.0;
    };

    let now = Utc::now();
    let age_days = (now - updated).num_days();

    if age_days < 0 {
        // Future date (shouldn't happen, but handle gracefully)
        return 0.0;
    }

    // Exponential decay with configurable thresholds
    if age_days <= 7 {
        0.5
    } else if age_days <= 30 {
        0.25
    } else if age_days <= 90 {
        0.1
    } else {
        0.0
    }
}

/// Check if ripgrep is available on the system
fn is_ripgrep_available() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn normalize_meta_path(store: &Store, meta_path: &str) -> String {
    let path = Path::new(meta_path);
    if path.is_absolute() {
        path.strip_prefix(store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    } else {
        meta_path.to_string()
    }
}

/// Search using ripgrep for faster file finding
///
/// This is an optimization that leverages ripgrep if available.
/// Falls back to embedded search if ripgrep is not found.
fn search_with_ripgrep(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let tokenized_query = tokenize(query);
    if tokenized_query.is_empty() {
        return Ok(Vec::new());
    }

    // Use ripgrep with JSON output to get both matches and context snippets
    let mut rg_cmd = Command::new("rg");
    rg_cmd
        .arg("--json")
        .arg("--case-insensitive")
        .arg("--no-heading")
        .arg("--with-filename")
        .arg("--context-before=1")
        .arg("--context-after=1")
        .arg("--max-columns=200");

    // Add search pattern (OR all terms together)
    let pattern = tokenized_query.join("|");
    rg_cmd.arg(&pattern);

    // Search in notes and mocs directories
    rg_cmd.arg(store.root().join("notes"));
    rg_cmd.arg(store.root().join("mocs"));

    let output = match rg_cmd.output() {
        Ok(output) => output,
        Err(_) => {
            // If ripgrep fails to run, fall back to embedded search
            return search_embedded(store, index, query, type_filter, tag_filter);
        }
    };

    // Parse ripgrep JSON output to get matches and contexts
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut matching_paths: HashSet<PathBuf> = HashSet::new();
    let mut path_contexts: HashMap<PathBuf, String> = HashMap::new();

    for line in stdout.lines() {
        if let Ok(rg_match) = serde_json::from_str::<RipgrepMatch>(line) {
            match rg_match {
                RipgrepMatch::Begin { path, .. } | RipgrepMatch::End { path, .. } => {
                    matching_paths.insert(PathBuf::from(path));
                }
                RipgrepMatch::Match { path, lines, .. } => {
                    let path_buf = PathBuf::from(&path);
                    matching_paths.insert(path_buf.clone());

                    // Store first context snippet for this file
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        path_contexts.entry(path_buf)
                    {
                        let context = lines.replace('\n', " ").trim().to_string();
                        if !context.is_empty() {
                            e.insert(format!("...{}...", context));
                        }
                    }
                }
            }
        }
    }

    // Also scan index metadata for title/tag matches that ripgrep might have missed
    // (e.g., if the term only appears in the YAML frontmatter title)
    let mut candidate_note_ids: HashSet<String> = HashSet::new();

    // Add note IDs from ripgrep file matches
    for path in &matching_paths {
        if let Some(entry) = index.files.get(path) {
            candidate_note_ids.insert(entry.note_id.clone());
        }
    }

    // Add note IDs that match in title or tags
    for (note_id, meta) in &index.metadata {
        let matches_title = tokenized_query
            .iter()
            .any(|t| meta.title.to_lowercase().contains(t));
        let matches_tags = tokenized_query
            .iter()
            .any(|t| meta.tags.iter().any(|tag| tag.to_lowercase().contains(t)));

        if matches_title || matches_tags {
            candidate_note_ids.insert(note_id.clone());
        }
    }

    // If no candidates found, use embedded search as fallback
    if candidate_note_ids.is_empty() {
        return search_embedded(store, index, query, type_filter, tag_filter);
    }

    // Build results from all candidate notes
    let mut results = Vec::new();
    let mut candidates_sorted: Vec<_> = candidate_note_ids.into_iter().collect();
    candidates_sorted.sort();

    for note_id in candidates_sorted {
        let meta = match index.metadata.get(&note_id) {
            Some(m) => m,
            None => continue,
        };

        // Apply type filter
        if let Some(t) = type_filter {
            if meta.note_type != t {
                continue;
            }
        }

        // Apply tag filter
        if let Some(tag) = tag_filter {
            if !meta.tags.contains(&tag.to_string()) {
                continue;
            }
        }

        // Read note to calculate BM25 score
        let note = match store.get_note_with_index(&meta.id, index) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let title_score = calculate_bm25(&tokenized_query, &meta.title, index, None);
        let tags_score = calculate_bm25(&tokenized_query, &meta.tags.join(" "), index, None);
        let body_score = calculate_bm25(
            &tokenized_query,
            &note.body,
            index,
            index.doc_lengths.get(&meta.id).copied(),
        );

        // Apply field boosting (Title x2.0, Tags x1.5) and recency boost
        let base_relevance = 2.0 * title_score + 1.5 * tags_score + body_score;
        let recency_boost = calculate_recency_boost(meta.updated);
        let relevance = base_relevance + recency_boost;

        // Only include results with some relevance
        if relevance > 0.0 {
            // Try to get context from ripgrep if available
            let path = PathBuf::from(&meta.path);
            let match_context = path_contexts.get(&path).cloned();

            results.push(SearchResult {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type,
                tags: meta.tags.clone(),
                path: normalize_meta_path(store, &meta.path),
                match_context,
                relevance,
                via: None,
            });
        }

        // Limit processing to improve performance for extremely large match sets
        if results.len() >= 500 {
            break;
        }
    }

    // Sort by relevance (descending), then by note ID (ascending) for deterministic tie-breaking
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Limit results to improve performance for large stores
    results.truncate(200);

    Ok(results)
}

/// Embedded text search (fallback when ripgrep not available)
fn search_embedded(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let tokenized_query = tokenize(query);
    if tokenized_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    // Iterate over metadata in deterministic order (sorted by note ID)
    let mut note_ids: Vec<&String> = index.metadata.keys().collect();
    note_ids.sort();
    for note_id in note_ids {
        let meta = &index.metadata[note_id];

        // Apply type filter
        if let Some(t) = type_filter {
            if meta.note_type != t {
                continue;
            }
        }

        // Apply tag filter
        if let Some(tag) = tag_filter {
            if !meta.tags.contains(&tag.to_string()) {
                continue;
            }
        }

        // Quick match check to avoid reading body of obviously irrelevant notes
        let matches_title_or_tags = tokenized_query.iter().any(|t| {
            meta.title.to_lowercase().contains(t)
                || meta.tags.iter().any(|tag| tag.to_lowercase().contains(t))
        });

        let matches_body = index
            .note_terms
            .get(note_id)
            .map(|terms| tokenized_query.iter().any(|t| terms.contains(t)))
            .unwrap_or(true);

        if !matches_title_or_tags && !matches_body {
            continue;
        }

        let title_score = calculate_bm25(&tokenized_query, &meta.title, index, None);
        let tags_score = calculate_bm25(&tokenized_query, &meta.tags.join(" "), index, None);

        let mut body_score = 0.0;
        let mut match_context = None;

        // Calculate body score (requires reading file)
        if let Ok(note) = store.get_note_with_index(&meta.id, index) {
            body_score = calculate_bm25(
                &tokenized_query,
                &note.body,
                index,
                index.doc_lengths.get(&meta.id).copied(),
            );

            // Extract context snippet if matched in body
            if body_score > 0.0 {
                let body_lower = note.body.to_lowercase();
                for term in &tokenized_query {
                    if let Some(pos) = body_lower.find(term) {
                        let start = pos.saturating_sub(40);
                        let end = (pos + term.len() + 40).min(note.body.len());
                        let snippet = &note.body[start..end];
                        match_context =
                            Some(format!("...{}...", snippet.replace('\n', " ").trim()));
                        break;
                    }
                }
            }
        }

        // Apply field boosting (Title x2.0, Tags x1.5) and recency boost
        let base_relevance = 2.0 * title_score + 1.5 * tags_score + body_score;
        let recency_boost = calculate_recency_boost(meta.updated);
        let relevance = base_relevance + recency_boost;

        if relevance > 0.0 {
            results.push(SearchResult {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type,
                tags: meta.tags.clone(),
                path: normalize_meta_path(store, &meta.path),
                match_context,
                relevance,
                via: None,
            });
        }
    }

    // Sort by relevance (descending), then by note ID (ascending) for deterministic tie-breaking
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Limit results to improve performance for large stores
    results.truncate(200);

    Ok(results)
}

/// Simple text search over the index
///
/// Uses ripgrep if available for faster file finding, otherwise falls back
/// to embedded matcher. Ranking: BM25 with field boosting (title > tags > body).
pub fn search(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    // Always try ripgrep first - it's much faster than embedded search
    if is_ripgrep_available() {
        if logging::verbose_enabled() {
            eprintln!("Using ripgrep search");
        }
        search_with_ripgrep(store, index, query, type_filter, tag_filter)
    } else {
        if logging::verbose_enabled() {
            eprintln!("Using embedded search");
        }
        search_embedded(store, index, query, type_filter, tag_filter)
    }
}
