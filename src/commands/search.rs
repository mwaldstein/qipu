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
use crate::lib::index::{search, Index, IndexBuilder, SearchResult};
use crate::lib::note::NoteType;
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
    // Load or build index (read-only - don't save)
    let cache_dir = store.root().join(".cache");
    let index = match Index::load(&cache_dir) {
        Ok(idx) if !idx.metadata.is_empty() => idx,
        _ => {
            // Index doesn't exist or is empty - build it in-memory
            if cli.verbose {
                eprintln!("Building index in-memory (run 'qipu index' to cache)...");
            }
            IndexBuilder::new(store).build()?
        }
    };

    let mut results = search(store, &index, query, type_filter, tag_filter)?;

    // Determine if compaction processing is needed
    let needs_compaction = !cli.no_resolve_compaction
        || cli.with_compaction_ids
        || cli.compaction_depth.is_some()
        || cli.compaction_max_nodes.is_some();

    // Only load all notes and build compaction context if needed
    let (all_notes, compaction_ctx) = if needs_compaction {
        let notes = store.list_notes()?;
        let ctx = CompactionContext::build(&notes)?;
        (notes, Some(ctx))
    } else {
        (Vec::new(), None)
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
                    // This is a compacted note - fetch the digest's metadata
                    if let Some(digest_meta) = index.get_metadata(&canonical_id) {
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
                                    if let Some(pct) = ctx.get_compaction_pct(note, &all_notes) {
                                        obj_mut.insert(
                                            "compaction_pct".to_string(),
                                            serde_json::json!(format!("{:.1}", pct)),
                                        );
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
                                if let Some(pct) = ctx.get_compaction_pct(note, &all_notes) {
                                    annotations.push_str(&format!(" compaction={:.0}%", pct));
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
                            if let Some(pct) = ctx.get_compaction_pct(note, &all_notes) {
                                annotations.push_str(&format!(" compaction={:.0}%", pct));
                            }
                        }
                    }
                }

                println!(
                    "N {} {} \"{}\" tags={}{}",
                    result.id, result.note_type, result.title, tags_csv, annotations
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
