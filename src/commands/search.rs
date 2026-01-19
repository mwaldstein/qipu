//! `qipu search` command - search notes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu search <query>` - search titles + bodies
//! - `--type` filter
//! - `--tag` filter
//! - Result ranking: title > exact tag > body, recency boost
//! - Compaction resolution (specs/compaction.md): show canonical digests with via= annotations

use std::collections::HashMap;

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::SearchResult;
use crate::lib::note::NoteType;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

/// Execute the search command
pub fn execute(
    cli: &Cli,
    store: &Store,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    exclude_mocs: bool,
) -> Result<()> {
    let mut results = store.db().search(query, type_filter, tag_filter, 200)?;

    // Determine if compaction processing is needed
    let needs_compaction = !cli.no_resolve_compaction
        || cli.with_compaction_ids
        || cli.compaction_depth.is_some()
        || cli.compaction_max_nodes.is_some();

    // Only load all notes and build compaction context if needed
    let all_notes = if needs_compaction {
        store.list_notes()?
    } else {
        Vec::new()
    };

    let compaction_ctx = if needs_compaction {
        Some(CompactionContext::build(&all_notes)?)
    } else {
        None
    };

    let compaction_note_map = if needs_compaction {
        Some(CompactionContext::build_note_map(&all_notes))
    } else {
        None
    };

    // Apply compaction resolution (unless --no-resolve-compaction)
    if !cli.no_resolve_compaction {
        if let Some(ref ctx) = compaction_ctx {
            // Group results by canonical ID, preserving the highest relevance and via field
            let mut canonical_results: HashMap<String, SearchResult> = HashMap::new();

            for mut result in results {
                let canonical_id = ctx.canon(&result.id)?;

                // If the note was compacted, we need to replace it with its digest
                if canonical_id != result.id {
                    // This is a compacted note - fetch the digest's metadata from DB
                    if let Some(digest_meta) = store.db().get_note_metadata(&canonical_id)? {
                        result.via = Some(result.id.clone());
                        result.id = canonical_id.clone();
                        result.title = digest_meta.title.clone();
                        result.note_type = digest_meta.note_type;
                        result.tags = digest_meta.tags.clone();
                        result.path = digest_meta.path.clone();
                    }
                }

                // Keep the highest relevance result for each canonical ID
                canonical_results
                    .entry(result.id.clone())
                    .and_modify(|existing| {
                        if result.relevance > existing.relevance {
                            *existing = result.clone();
                        } else if result.via.is_some() && existing.via.is_none() {
                            // Prefer keeping the via field if we have it
                            existing.via = result.via.clone();
                        }
                    })
                    .or_insert(result);
            }

            // Convert HashMap to Vec in deterministic order (sorted by note ID)
            let mut sorted_entries: Vec<_> = canonical_results.into_iter().collect();
            sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));
            results = sorted_entries
                .into_iter()
                .map(|(_, result)| result)
                .collect();

            // Re-sort by relevance after canonicalization, then by note ID for deterministic tie-breaking
            results.sort_by(|a, b| {
                b.relevance
                    .partial_cmp(&a.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.id.cmp(&b.id))
            });
        }
    }

    // Apply exclude_mocs filter if requested
    if exclude_mocs {
        results.retain(|r| r.note_type != NoteType::Moc);
    }

    // Pre-load notes for compaction annotations to avoid repeated I/O
    // Only load notes that are actually in results and have compaction info
    let mut notes_cache: HashMap<String, crate::lib::note::Note> = HashMap::new();
    if let Some(ref ctx) = compaction_ctx {
        for result in &results {
            let compacts_count = ctx.get_compacts_count(&result.id);
            if compacts_count > 0 && !notes_cache.contains_key(&result.id) {
                if let Ok(note) = store.get_note(&result.id) {
                    notes_cache.insert(result.id.clone(), note);
                }
            }
        }
    }

    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = results
                .iter()
                .map(|r| {
                    let mut obj = serde_json::json!({
                        "id": r.id,
                        "title": r.title,
                        "type": r.note_type.to_string(),
                        "tags": r.tags,
                        "path": r.path,
                        "match_context": r.match_context,
                        "relevance": r.relevance,
                    });
                    // Add via field if present (per spec: specs/compaction.md line 122)
                    if let Some(via) = &r.via {
                        if let Some(obj_mut) = obj.as_object_mut() {
                            obj_mut.insert("via".to_string(), serde_json::json!(via));
                        }
                    }

                    // Add compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    if let Some(ref ctx) = compaction_ctx {
                        let compacts_count = ctx.get_compacts_count(&r.id);
                        if compacts_count > 0 {
                            if let Some(obj_mut) = obj.as_object_mut() {
                                obj_mut.insert(
                                    "compacts".to_string(),
                                    serde_json::json!(compacts_count),
                                );

                                // For compaction_pct, use cached note to avoid repeated I/O
                                if let Some(note) = notes_cache.get(&r.id) {
                                    if let Some(ref note_map) = compaction_note_map {
                                        if let Some(pct) = ctx.get_compaction_pct(note, note_map) {
                                            obj_mut.insert(
                                                "compaction_pct".to_string(),
                                                serde_json::json!(format!("{:.1}", pct)),
                                            );
                                        }
                                    }
                                }

                                // Add compacted IDs if --with-compaction-ids is set
                                // Per spec (specs/compaction.md line 131)
                                if cli.with_compaction_ids {
                                    let depth = cli.compaction_depth.unwrap_or(1);
                                    if let Some((ids, _truncated)) = ctx.get_compacted_ids(
                                        &r.id,
                                        depth,
                                        cli.compaction_max_nodes,
                                    ) {
                                        obj_mut.insert(
                                            "compacted_ids".to_string(),
                                            serde_json::json!(ids),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    obj
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if results.is_empty() {
                if !cli.quiet {
                    println!("No results found for '{}'", query);
                }
            } else {
                for result in &results {
                    let type_indicator = match result.note_type {
                        NoteType::Fleeting => "F",
                        NoteType::Literature => "L",
                        NoteType::Permanent => "P",
                        NoteType::Moc => "M",
                    };

                    // Build annotations
                    let mut annotations = String::new();

                    // Add via annotation if present (per spec: specs/compaction.md line 122)
                    if let Some(via) = &result.via {
                        annotations.push_str(&format!(" (via {})", via));
                    }

                    // Add compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    let mut compacts_count = 0;
                    if let Some(ref ctx) = compaction_ctx {
                        compacts_count = ctx.get_compacts_count(&result.id);
                        if compacts_count > 0 {
                            annotations.push_str(&format!(" compacts={}", compacts_count));

                            // For compaction_pct, use cached note to avoid repeated I/O
                            if let Some(note) = notes_cache.get(&result.id) {
                                if let Some(ref note_map) = compaction_note_map {
                                    if let Some(pct) = ctx.get_compaction_pct(note, note_map) {
                                        annotations.push_str(&format!(" compaction={:.0}%", pct));
                                    }
                                }
                            }
                        }
                    }

                    println!(
                        "{} [{}] {}{}",
                        result.id, type_indicator, result.title, annotations
                    );
                    if cli.verbose {
                        if let Some(ctx) = &result.match_context {
                            println!("    {}", ctx);
                        }
                    }

                    // Show compacted IDs if --with-compaction-ids is set
                    // Per spec (specs/compaction.md line 131)
                    if cli.with_compaction_ids && compacts_count > 0 {
                        if let Some(ref ctx) = compaction_ctx {
                            let depth = cli.compaction_depth.unwrap_or(1);
                            if let Some((ids, truncated)) =
                                ctx.get_compacted_ids(&result.id, depth, cli.compaction_max_nodes)
                            {
                                let ids_str = ids.join(", ");
                                let suffix = if truncated {
                                    let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                                    format!(" (truncated, showing {} of {})", max, compacts_count)
                                } else {
                                    String::new()
                                };
                                println!("  Compacted: {}{}", ids_str, suffix);
                            }
                        }
                    }
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=search query=\"{}\" results={}",
                store.root().display(),
                query.replace('"', "\\\""),
                results.len()
            );
            for result in &results {
                let tags_csv = if result.tags.is_empty() {
                    "-".to_string()
                } else {
                    result.tags.join(",")
                };

                // Build annotations
                let mut annotations = String::new();

                // Add via field if present (per spec: specs/compaction.md line 122)
                if let Some(via) = &result.via {
                    annotations.push_str(&format!(" via={}", via));
                }

                // Add compaction annotations for digest notes
                // Per spec (specs/compaction.md lines 116-119, 125)
                let mut compacts_count = 0;
                if let Some(ref ctx) = compaction_ctx {
                    compacts_count = ctx.get_compacts_count(&result.id);
                    if compacts_count > 0 {
                        annotations.push_str(&format!(" compacts={}", compacts_count));

                        // For compaction_pct, use cached note to avoid repeated I/O
                        if let Some(note) = notes_cache.get(&result.id) {
                            if let Some(ref note_map) = compaction_note_map {
                                if let Some(pct) = ctx.get_compaction_pct(note, note_map) {
                                    annotations.push_str(&format!(" compaction={:.0}%", pct));
                                }
                            }
                        }
                    }
                }

                println!(
                    "N {} {} \"{}\" tags={}{}",
                    result.id,
                    result.note_type,
                    escape_quotes(&result.title),
                    tags_csv,
                    annotations
                );
                if let Some(ctx) = &result.match_context {
                    println!("S {} {}", result.id, ctx);
                }

                // Show compacted IDs if --with-compaction-ids is set
                // Per spec (specs/compaction.md line 131)
                if cli.with_compaction_ids && compacts_count > 0 {
                    if let Some(ref ctx) = compaction_ctx {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&result.id, depth, cli.compaction_max_nodes)
                        {
                            for id in &ids {
                                println!("D compacted {} from={}", id, result.id);
                            }
                            if truncated {
                                println!(
                                    "D compacted_truncated max={} total={}",
                                    cli.compaction_max_nodes.unwrap_or(ids.len()),
                                    compacts_count
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::lib::note::NoteType;
    use crate::lib::store::InitOptions;
    use tempfile::tempdir;

    #[test]
    fn test_search_empty_query() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            no_resolve_compaction: false,
            with_compaction_ids: false,
            compaction_depth: None,
            compaction_max_nodes: None,
            ..Default::default()
        };

        let result = execute(&cli, &store, "", None, None, false);
        assert!(result.is_ok(), "Empty query should not error");
    }

    #[test]
    fn test_search_no_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "nonexistent", None, None, false);
        assert!(result.is_ok(), "Search with no results should succeed");
    }

    #[test]
    fn test_search_with_type_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Permanent Note", Some(NoteType::Permanent), &[], None)
            .unwrap();
        store
            .create_note("Fleeting Note", Some(NoteType::Fleeting), &[], None)
            .unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "note", Some(NoteType::Permanent), None, false);
        assert!(result.is_ok(), "Search with type filter should succeed");
    }

    #[test]
    fn test_search_with_tag_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Tagged Note", None, &["rust".to_string()], None)
            .unwrap();
        store.create_note("Untagged Note", None, &[], None).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "note", None, Some("rust"), false);
        assert!(result.is_ok(), "Search with tag filter should succeed");
    }

    #[test]
    fn test_search_exclude_mocs() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("MOC Note", Some(NoteType::Moc), &[], None)
            .unwrap();
        store
            .create_note("Regular Note", Some(NoteType::Fleeting), &[], None)
            .unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "note", None, None, true);
        assert!(result.is_ok(), "Search with MOC exclusion should succeed");
    }

    #[test]
    fn test_search_json_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["test".to_string()], None)
            .unwrap();

        let cli = Cli {
            format: OutputFormat::Json,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "test", None, None, false);
        assert!(result.is_ok(), "Search with JSON format should succeed");
    }

    #[test]
    fn test_search_records_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["test".to_string()], None)
            .unwrap();

        let cli = Cli {
            format: OutputFormat::Records,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "test", None, None, false);
        assert!(result.is_ok(), "Search with records format should succeed");
    }

    #[test]
    fn test_search_quiet_no_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: true,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "nonexistent", None, None, false);
        assert!(
            result.is_ok(),
            "Quiet search with no results should succeed"
        );
    }

    #[test]
    fn test_search_verbose_output() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Test Note", None, &[], None).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: true,
            ..Default::default()
        };

        let result = execute(&cli, &store, "test", None, None, false);
        assert!(result.is_ok(), "Verbose search should succeed");
    }

    #[test]
    fn test_search_compaction_resolution() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store.create_note("Digest Note", None, &[], None).unwrap();
        note1.body = "This is the digest content.\n\nCompacts from qp-abc, qp-def".to_string();
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.create_note("Source Note", None, &[], None).unwrap();
        note2.body = "This will be compacted into qp-digest".to_string();
        store.save_note(&mut note2).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "digest", None, None, false);
        assert!(
            result.is_ok(),
            "Search with compaction resolution should succeed"
        );
    }

    #[test]
    fn test_search_no_resolve_compaction() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Test Note", None, &[], None).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            no_resolve_compaction: true,
            ..Default::default()
        };

        let result = execute(&cli, &store, "test", None, None, false);
        assert!(
            result.is_ok(),
            "Search without compaction resolution should succeed"
        );
    }

    #[test]
    fn test_search_with_compaction_ids() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store.create_note("Digest Note", None, &[], None).unwrap();
        note1.body = "Digest content\n\nCompacts from qp-source".to_string();
        store.save_note(&mut note1).unwrap();

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            with_compaction_ids: true,
            compaction_depth: Some(1),
            ..Default::default()
        };

        let result = execute(&cli, &store, "digest", None, None, false);
        assert!(result.is_ok(), "Search with compaction IDs should succeed");
    }

    #[test]
    fn test_search_multiple_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        for i in 0..5 {
            store
                .create_note(&format!("Note {}", i), None, &[], None)
                .unwrap();
        }

        let cli = Cli {
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            ..Default::default()
        };

        let result = execute(&cli, &store, "note", None, None, false);
        assert!(
            result.is_ok(),
            "Search with multiple results should succeed"
        );
    }
}
