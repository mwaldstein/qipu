//! `qipu context` command - build context bundles for LLM integration
//!
//! Per spec (specs/llm-context.md):
//! - `qipu context` outputs a bundle of notes designed for LLM context injection
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Budgeting: `--max-chars` exact budget
//! - Formats: human (markdown), json, records
//! - Safety: notes are untrusted inputs, optional safety banner

pub mod budget;
pub mod filter;
pub mod human;
pub mod json;
pub mod output;
pub mod records;
pub mod select;
pub mod types;
pub mod walk;

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::IndexBuilder;
use crate::lib::similarity::SimilarityEngine;
use crate::lib::store::Store;

pub use types::ContextOptions;
use types::{RecordsOutputConfig, SelectedNote};

use filter::parse_custom_filter_expression;

/// Convert an absolute path to a path relative to the current working directory
pub fn path_relative_to_cwd(path: &std::path::Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd)
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

/// Execute the context command
pub fn execute(cli: &Cli, store: &Store, options: ContextOptions) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        tracing::debug!(
            note_ids_count = options.note_ids.len(),
            tag = options.tag,
            moc_id = options.moc_id,
            query = options.query,
            max_chars = options.max_chars,
            transitive = options.transitive,
            with_body = options.with_body,
            safety_banner = options.safety_banner,
            related_threshold = options.related_threshold,
            backlinks = options.backlinks,
            min_value = options.min_value,
            "context_params"
        );
    }

    // Build compaction context for annotations
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(&all_notes);

    // Collect notes based on selection criteria
    let mut selected_notes: Vec<SelectedNote> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut via_map: HashMap<String, String> = HashMap::new();

    let resolve_id = |id: &str| -> Result<String> {
        if cli.no_resolve_compaction {
            Ok(id.to_string())
        } else {
            compaction_ctx.canon(id)
        }
    };

    // Selection by graph walk
    if let Some(walk_id) = options.walk_id {
        let walked_ids = walk::walk_for_context(
            cli,
            store,
            walk_id,
            options.walk_direction,
            options.walk_max_hops,
            options.walk_type,
            options.walk_exclude_type,
            options.walk_typed_only,
            options.walk_inline_only,
            options.walk_max_nodes,
            options.walk_max_edges,
            options.walk_max_fanout,
            options.walk_min_value,
            options.walk_ignore_value,
        )?;

        for id in &walked_ids {
            let resolved_id = resolve_id(id)?;

            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: Some(format!("walk:{}", walk_id)),
                    link_type: None,
                });
            }
        }
    }

    // Selection by explicit note IDs
    for id in options.note_ids {
        let resolved_id = resolve_id(id)?;

        if seen_ids.insert(resolved_id.clone()) {
            let note =
                note_map
                    .get(resolved_id.as_str())
                    .ok_or_else(|| QipuError::NoteNotFound {
                        id: resolved_id.clone(),
                    })?;
            selected_notes.push(SelectedNote {
                note,
                via: None,
                link_type: None,
            });
        }
    }

    // Selection by tag
    if let Some(tag_name) = options.tag {
        let notes_with_tag = store.db().list_notes(None, Some(tag_name), None)?;
        for note in &notes_with_tag {
            let resolved_id = resolve_id(&note.id)?;

            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    // Selection by MOC
    if let Some(moc) = options.moc_id {
        let linked_ids = select::get_moc_linked_ids(store.db(), moc, options.transitive)?;
        for (id, link_type) in linked_ids {
            let resolved_id = resolve_id(&id)?;

            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type,
                });
            }
        }
    }

    // Selection by query
    if let Some(q) = options.query {
        let results = store.db().search(q, None, None, None, None, 100)?;
        for result in results {
            let resolved_id = resolve_id(&result.id)?;
            let via_source = if !cli.no_resolve_compaction && resolved_id != result.id {
                Some(result.id.clone())
            } else {
                None
            };

            if let Some(via_id) = via_source {
                via_map.entry(resolved_id.clone()).or_insert(via_id);
            }

            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    // Backlink expansion
    if options.backlinks {
        let mut backlink_notes: Vec<(String, String, crate::lib::note::LinkType)> = Vec::new();

        for selected_note in &selected_notes {
            let note_id = selected_note.note.id();
            let backlinks = store.db().get_backlinks(note_id)?;

            for backlink in backlinks {
                if !seen_ids.contains(&backlink.from) {
                    backlink_notes.push((backlink.from, note_id.to_string(), backlink.link_type));
                }
            }
        }

        for (backlink_id, source_id, link_type) in backlink_notes {
            let resolved_id = resolve_id(&backlink_id)?;
            via_map
                .entry(resolved_id.clone())
                .or_insert_with(|| format!("backlink:{}", source_id));
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: Some(link_type),
                });
            }
        }
    }

    // Similarity-based expansion
    if let Some(threshold) = options.related_threshold {
        use std::time::Instant;
        let start = Instant::now();
        let index = IndexBuilder::new(store).build()?;

        if cli.verbose {
            tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
        }

        let engine = SimilarityEngine::new(&index);

        // Collect linked IDs to exclude them
        let mut linked_ids: HashSet<String> = HashSet::new();
        for selected_note in &selected_notes {
            let note_id = selected_note.note.id();
            let outbound_edges = index.get_outbound_edges(note_id);
            for edge in outbound_edges {
                linked_ids.insert(edge.to.clone());
            }
            let inbound_edges = index.get_inbound_edges(note_id);
            for edge in inbound_edges {
                linked_ids.insert(edge.from.clone());
            }
        }

        // Find related notes using multiple methods
        let mut related_notes: Vec<(String, f64, String)> = Vec::new();

        for selected_note in &selected_notes {
            let note_id = selected_note.note.id();

            // Method 1: TF-IDF similarity
            let similar = engine.find_similar(note_id, 100, threshold);
            for sim in similar {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "similarity".to_string()));
                }
            }

            // Method 2: Shared tags
            let shared_tags = engine.find_by_shared_tags(note_id, 100);
            for sim in shared_tags {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "shared-tags".to_string()));
                }
            }

            // Method 3: 2-hop neighborhoods
            let two_hop = engine.find_by_2hop_neighborhood(note_id, 100);
            for sim in two_hop {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "2hop".to_string()));
                }
            }
        }

        // Deduplicate and sort by score
        related_notes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap()));
        related_notes.dedup_by(|a, b| a.0 == b.0);
        related_notes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Add related notes to selection
        for (related_id, score, method) in related_notes {
            if seen_ids.contains(&related_id) {
                continue;
            }
            let resolved_id = resolve_id(&related_id)?;
            // Mark as added via relatedness
            via_map
                .entry(resolved_id.clone())
                .or_insert_with(|| format!("{}:{:.2}", method, score));
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: Some(related_id.clone()),
                    link_type: None,
                });
            }
        }
    }

    for selected in &mut selected_notes {
        if let Some(via) = via_map.get(selected.note.id()) {
            selected.via = Some(via.clone());
        }
    }

    // If no selection criteria provided, check for standalone filters
    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
        && options.walk_id.is_none()
    {
        // Check if min-value or custom-filter is provided as a standalone selector
        if options.min_value.is_none() && options.custom_filter.is_empty() {
            return Err(QipuError::UsageError(
                "no selection criteria provided. Use --note, --tag, --moc, --query, --walk, --min-value, or --custom-filter".to_string(),
            ));
        }

        // Select all notes when using standalone filter
        for note in &all_notes {
            let resolved_id = resolve_id(note.id())?;

            if seen_ids.insert(resolved_id.clone()) {
                let note_from_map =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note: note_from_map,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    // Apply min-value filter (notes without explicit value default to 50)
    if let Some(min_value) = options.min_value {
        let before_count = selected_notes.len();
        selected_notes.retain(|selected| {
            let note_value = selected.note.frontmatter.value.unwrap_or(50);
            note_value >= min_value
        });
        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            tracing::debug!(
                min_value,
                before_count,
                after_count,
                filtered = before_count - after_count,
                "min_value_filter"
            );
        }
    }

    // Apply custom metadata filters
    if !options.custom_filter.is_empty() {
        let before_count = selected_notes.len();

        // Parse filter expressions
        let filters: Vec<
            std::sync::Arc<dyn Fn(&std::collections::HashMap<String, serde_yaml::Value>) -> bool>,
        > = options
            .custom_filter
            .iter()
            .map(|filter_expr| parse_custom_filter_expression(filter_expr))
            .collect::<Result<_>>()?;

        // Apply all filters with AND semantics (retain only notes that match all filters)
        selected_notes.retain(|selected| {
            filters
                .iter()
                .all(|filter| filter(&selected.note.frontmatter.custom))
        });

        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            tracing::debug!(
                filter_count = options.custom_filter.len(),
                before_count,
                after_count,
                filtered = before_count - after_count,
                "custom_filters"
            );
        }
    }

    // Sort notes: Prioritize verified notes, then typed links (especially part-of and supports) over related, then by created date, then by id
    selected_notes.sort_by(|a, b| {
        // b.cmp(a) for verified because we want true (1) before false (0)
        let a_verified = a.note.frontmatter.verified.unwrap_or(false);
        let b_verified = b.note.frontmatter.verified.unwrap_or(false);

        // Helper function to get link priority: lower number = higher priority
        let link_priority = |link_type: &Option<crate::lib::note::LinkType>| -> u8 {
            match link_type {
                Some(lt) if lt.as_str() == "part-of" || lt.as_str() == "supports" => 0,
                Some(lt) if lt.as_str() != "related" => 1,
                Some(_) => 2, // related
                None => 1,    // directly selected, same priority as other typed links
            }
        };

        let a_priority = link_priority(&a.link_type);
        let b_priority = link_priority(&b.link_type);

        b_verified
            .cmp(&a_verified)
            .then_with(|| a_priority.cmp(&b_priority))
            .then_with(
                || match (&a.note.frontmatter.created, &b.note.frontmatter.created) {
                    (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
            )
            .then_with(|| a.note.id().cmp(b.note.id()))
    });

    // Apply budgeting (records format handles its own exact budget)
    let (truncated, notes_to_output, excluded_notes) = match cli.format {
        OutputFormat::Records => (false, selected_notes.iter().collect(), Vec::new()),
        _ => budget::apply_budget(&selected_notes, options.max_chars, options.with_body),
    };

    // Output in requested format (paths are relative to current working directory per spec)
    let store_path = path_relative_to_cwd(store.root());

    match cli.format {
        OutputFormat::Json => {
            output::output_json(
                cli,
                &store_path,
                &notes_to_output,
                truncated,
                options.with_body,
                &compaction_ctx,
                &note_map,
                &all_notes,
                options.max_chars,
                &excluded_notes,
                options.include_custom,
            )?;
        }
        OutputFormat::Human => {
            output::output_human(
                cli,
                &store_path,
                &notes_to_output,
                truncated,
                options.with_body,
                options.safety_banner,
                &compaction_ctx,
                &note_map,
                &all_notes,
                options.max_chars,
                &excluded_notes,
                options.include_custom,
            );
        }
        OutputFormat::Records => {
            let config = RecordsOutputConfig {
                truncated,
                with_body: options.with_body,
                safety_banner: options.safety_banner,
                max_chars: options.max_chars,
            };
            output::output_records(
                cli,
                &store_path,
                &notes_to_output,
                &config,
                &compaction_ctx,
                &note_map,
                &all_notes,
                options.include_custom,
            );
        }
    }

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), notes_output = notes_to_output.len(), truncated, "context_complete");
    }

    Ok(())
}
