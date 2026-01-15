use super::types::{Index, SearchResult};
use crate::lib::error::Result;
use crate::lib::logging;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
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

/// Check if ripgrep is available on the system
fn is_ripgrep_available() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn absolute_meta_path(store: &Store, meta_path: &str) -> PathBuf {
    let path = Path::new(meta_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        store.root().join(path)
    }
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
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

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
    let pattern = query_terms.join("|");
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

    // If ripgrep found no matches, use embedded search as fallback
    // (ripgrep exit code 1 means no matches, not an error)
    if matching_paths.is_empty() {
        return search_embedded(store, index, query, type_filter, tag_filter);
    }

    // Build results from matching files using index metadata
    let mut results = Vec::new();

    // For performance, limit processing to first 200 matching files
    // Users rarely look beyond the first few pages of results
    let mut processed = 0;
    const MAX_FILES_TO_PROCESS: usize = 200;

    // Create a faster lookup for matching paths
    let matching_path_set: HashSet<&PathBuf> = matching_paths.iter().collect();

    // Iterate over metadata in deterministic order (sorted by note ID)
    let mut note_ids: Vec<&String> = index.metadata.keys().collect();
    note_ids.sort();
    for note_id in note_ids {
        let meta = &index.metadata[note_id];
        // Skip if path doesn't match ripgrep results
        let path = absolute_meta_path(store, &meta.path);
        if !matching_path_set.contains(&path) {
            continue;
        }

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

        // Calculate relevance score (same logic as embedded search) - optimized
        let mut relevance = 0.0;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight) - optimized early exit
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                if title_lower == *term {
                    relevance += 5.0;
                }
                // Early exit for strong title matches
                if relevance >= 15.0 {
                    break;
                }
            }
        }

        // Tag matches (medium weight) - only check if needed
        if relevance < 15.0 {
            for tag in &meta.tags {
                let tag_lower = tag.to_lowercase();
                for term in &query_terms {
                    if tag_lower == *term {
                        relevance += 7.0;
                    } else if tag_lower.contains(term) {
                        relevance += 3.0;
                    }
                }
                if relevance >= 10.0 {
                    break;
                }
            }
        }

        // Use pre-fetched context and add relevance for body matches
        if let Some(context) = path_contexts.get(&path) {
            // We know this file matched in ripgrep, so add body relevance
            for term in &query_terms {
                if context.to_lowercase().contains(term) {
                    relevance += 2.0;
                }
            }
            match_context = Some(context.clone());
        }

        // Recency boost (prefer recently updated notes)
        let timestamp = meta.updated.or(meta.created);
        if let Some(ts) = timestamp {
            let age_days = (Utc::now() - ts).num_days();
            if age_days < 7 {
                relevance += 1.0;
            } else if age_days < 30 {
                relevance += 0.5;
            }
        }

        // Only include results with some relevance
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

        processed += 1;
        if processed >= MAX_FILES_TO_PROCESS {
            break;
        }

        // Early exit if we have enough strong results
        if results.len() >= 50 {
            let strong_count = results.iter().filter(|r| r.relevance >= 10.0).count();
            if strong_count >= 20 {
                break;
            }
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
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let tag_string = tag_filter.map(|t| t.to_string());

    // Early exit: if we have strong title matches in metadata, limit body reads
    let mut strong_title_matches = 0;
    const MAX_STRONG_MATCHES: usize = 50;

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
        if let Some(ref tag) = tag_string {
            if !meta.tags.contains(tag) {
                continue;
            }
        }

        // Calculate relevance score
        let mut relevance = 0.0;
        let mut matched = false;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight) - optimized to avoid repeated contains() calls
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                matched = true;
                // Exact title match bonus
                if title_lower == *term {
                    relevance += 5.0;
                }
                // Break early if we have a strong title match to avoid unnecessary body reads
                if relevance >= 15.0 {
                    strong_title_matches += 1;
                    break;
                }
            }
        }

        // Skip body search if we already have enough strong matches
        if strong_title_matches >= MAX_STRONG_MATCHES && relevance >= 15.0 {
            matched = true; // Ensure we include this result
        } else {
            // Tag matches (medium weight) - optimized to avoid repeated to_lowercase()
            if !matched || relevance < 15.0 {
                for tag in &meta.tags {
                    let tag_lower = tag.to_lowercase();
                    for term in &query_terms {
                        if tag_lower == *term {
                            relevance += 7.0;
                            matched = true;
                        } else if tag_lower.contains(term) {
                            relevance += 3.0;
                            matched = true;
                        }
                    }
                    if relevance >= 10.0 {
                        break;
                    }
                }
            }

            // Body search (lower weight, requires reading file) - only if needed and under limit
            if !matched || relevance < 10.0 {
                // Read note content to search body (use index for fast path lookup)
                if let Ok(note) = store.get_note_with_index(&meta.id, index) {
                    let body_lower = note.body.to_lowercase();
                    for term in &query_terms {
                        if body_lower.contains(term) {
                            relevance += 2.0;
                            matched = true;

                            // Extract context snippet - only for first match
                            if match_context.is_none() {
                                if let Some(pos) = body_lower.find(term) {
                                    let start = pos.saturating_sub(40);
                                    let end = (pos + term.len() + 40).min(note.body.len());
                                    let snippet = &note.body[start..end];
                                    let snippet = snippet.replace('\n', " ");
                                    match_context = Some(format!("...{}...", snippet.trim()));
                                    break; // Only get context for first term match
                                }
                            }
                        }
                    }
                }
            }
        }

        // Recency boost (prefer recently updated notes)
        if matched {
            let timestamp = meta.updated.or(meta.created);
            if let Some(ts) = timestamp {
                let age_days = (Utc::now() - ts).num_days();
                if age_days < 7 {
                    relevance += 1.0;
                } else if age_days < 30 {
                    relevance += 0.5;
                }
            }

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
/// to embedded matcher. Ranking: title matches > exact tag matches > body matches,
/// with recency boost.
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
