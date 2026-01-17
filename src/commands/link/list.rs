//! Link list command
use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Index, IndexBuilder};
use crate::lib::store::Store;

use super::{
    filter_and_convert, filter_and_convert_inbound, resolve_note_id, Direction, LinkEntry,
};

/// Execute the link list command
///
/// Lists all links for a note, with optional direction and type filters.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    direction: Direction,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
    max_chars: Option<usize>,
) -> Result<()> {
    // Resolve the note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build the index
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    let all_notes = store.list_notes()?;

    // Build compaction context if needed
    let compaction_ctx = if !cli.no_resolve_compaction {
        Some(CompactionContext::build(&all_notes)?)
    } else {
        None
    };

    let equivalence_map = if let Some(ref ctx) = compaction_ctx {
        Some(ctx.build_equivalence_map(&all_notes)?)
    } else {
        None
    };

    // Canonicalize the note ID to get which note's links we should show
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    let display_id = if compaction_ctx.is_some() {
        canonical_id.clone()
    } else {
        note_id.clone()
    };

    // Verify canonical note exists
    if !index.contains(&canonical_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Collect all raw IDs that map to this canonical ID (for gathering edges)
    let source_ids = equivalence_map
        .as_ref()
        .and_then(|map| map.get(&canonical_id).cloned())
        .unwrap_or_else(|| vec![canonical_id.clone()]);

    // Collect links based on direction
    let mut entries = Vec::new();

    // Outbound edges (links FROM this note or any note it compacts)
    if direction == Direction::Out || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_outbound_edges(source_id) {
                if let Some(mut entry) =
                    filter_and_convert(edge, "out", &index, type_filter, typed_only, inline_only)
                {
                    // Canonicalize the target ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        if entry.id == canonical_id {
                            continue;
                        }
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Inbound edges (backlinks TO this note or any note it compacts)
    if direction == Direction::In || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_inbound_edges(source_id) {
                if let Some(mut entry) = filter_and_convert_inbound(
                    edge,
                    &index,
                    store,
                    type_filter,
                    typed_only,
                    inline_only,
                    !cli.no_semantic_inversion,
                ) {
                    // Canonicalize the source ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        if entry.id == canonical_id {
                            continue;
                        }
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Remove duplicates that may have been created by canonicalization
    entries.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.id.cmp(&b.id))
    });
    entries
        .dedup_by(|a, b| a.direction == b.direction && a.link_type == b.link_type && a.id == b.id);

    // Output
    match cli.format {
        OutputFormat::Json => {
            let json_output: Vec<serde_json::Value> = entries
                .iter()
                .map(|entry| {
                    let mut json = serde_json::json!({
                        "direction": entry.direction,
                        "id": entry.id,
                        "type": entry.link_type,
                        "source": entry.source,
                    });
                    if let Some(title) = &entry.title {
                        if let Some(obj_mut) = json.as_object_mut() {
                            obj_mut.insert("title".to_string(), serde_json::json!(title));
                        }
                    }

                    // Add compacted IDs if --with-compaction-ids is set
                    if cli.with_compaction_ids {
                        if let Some(ref ctx) = compaction_ctx {
                            let compacts_count = ctx.get_compacts_count(&entry.id);
                            if compacts_count > 0 {
                                let depth = cli.compaction_depth.unwrap_or(1);
                                if let Some((ids, _truncated)) = ctx.get_compacted_ids(
                                    &entry.id,
                                    depth,
                                    cli.compaction_max_nodes,
                                ) {
                                    if let Some(obj_mut) = json.as_object_mut() {
                                        obj_mut.insert(
                                            "compacted_ids".to_string(),
                                            serde_json::json!(ids),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    json
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        OutputFormat::Human => {
            if entries.is_empty() {
                if !cli.quiet {
                    println!("No links found for {}", display_id);
                }
            } else {
                for entry in &entries {
                    let dir_arrow = match entry.direction.as_str() {
                        "out" => "->",
                        "in" => "<-",
                        _ => "--",
                    };
                    let title_part = entry
                        .title
                        .as_ref()
                        .map(|t| format!(" \"{}\"", t))
                        .unwrap_or_default();
                    println!(
                        "{} {} {} [{}] ({})",
                        dir_arrow, entry.id, title_part, entry.link_type, entry.source
                    );

                    // Show compacted IDs if --with-compaction-ids is set
                    if cli.with_compaction_ids {
                        if let Some(ref ctx) = compaction_ctx {
                            let compacts_count = ctx.get_compacts_count(&entry.id);
                            if compacts_count > 0 {
                                let depth = cli.compaction_depth.unwrap_or(1);
                                if let Some((ids, truncated)) = ctx.get_compacted_ids(
                                    &entry.id,
                                    depth,
                                    cli.compaction_max_nodes,
                                ) {
                                    let ids_str = ids.join(", ");
                                    let suffix = if truncated {
                                        let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                                        format!(
                                            " (truncated, showing {} of {})",
                                            max, compacts_count
                                        )
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
        }
        OutputFormat::Records => {
            output_list_records(
                &entries,
                store,
                &index,
                &display_id,
                direction,
                cli,
                compaction_ctx.as_ref(),
                max_chars,
            );
        }
    }

    Ok(())
}

fn output_list_records(
    entries: &[LinkEntry],
    store: &Store,
    index: &Index,
    display_id: &str,
    direction: Direction,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    max_chars: Option<usize>,
) {
    let mut lines = Vec::new();

    let mut unique_ids: Vec<String> = entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    unique_ids.sort();

    for link_id in &unique_ids {
        if let Some(meta) = index.get_metadata(link_id) {
            let tags_csv = if meta.tags.is_empty() {
                "-".to_string()
            } else {
                meta.tags.join(",")
            };
            lines.push(format!(
                "N {} {} \"{}\" tags={}",
                link_id, meta.note_type, meta.title, tags_csv
            ));

            if let Ok(note) = store.get_note(link_id) {
                let summary = note.summary();
                if !summary.is_empty() {
                    let summary_text = summary.lines().next().unwrap_or("").trim();
                    if !summary_text.is_empty() {
                        lines.push(format!("S {} {}", link_id, summary_text));
                    }
                }
            }

            if cli.with_compaction_ids {
                if let Some(ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(link_id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(link_id, depth, cli.compaction_max_nodes)
                        {
                            for id in &ids {
                                lines.push(format!("D compacted {} from={}", id, link_id));
                            }
                            if truncated {
                                lines.push(format!(
                                    "D compacted_truncated max={} total={}",
                                    cli.compaction_max_nodes.unwrap_or(ids.len()),
                                    compacts_count
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    for entry in entries {
        let (from, to) = match entry.direction.as_str() {
            "out" => (display_id.to_string(), entry.id.clone()),
            "in" => (entry.id.clone(), display_id.to_string()),
            _ => (display_id.to_string(), entry.id.clone()),
        };
        lines.push(format!(
            "E {} {} {} {}",
            from, entry.link_type, to, entry.source
        ));
    }

    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.list id={} direction={} truncated=",
        store.root().display(),
        display_id,
        match direction {
            Direction::Out => "out",
            Direction::In => "in",
            Direction::Both => "both",
        }
    );
    let header_len_false = header_base.len() + "false".len() + 1;
    let header_len_true = header_base.len() + "true".len() + 1;

    fn select_lines(header_len: usize, budget: Option<usize>, lines: &[String]) -> (bool, usize) {
        if let Some(max) = budget {
            if header_len > max {
                return (true, 0);
            }
        }

        let mut used = header_len;
        let mut count = 0;
        for line in lines {
            let line_len = line.len() + 1;
            if budget.map_or(true, |max| used + line_len <= max) {
                used += line_len;
                count += 1;
            } else {
                return (true, count);
            }
        }

        (false, count)
    }

    let (budget_truncated, line_count, truncated) = {
        let (budget_flag, count) = select_lines(header_len_false, max_chars, &lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, max_chars, &lines);
            (budget_flag, count, true)
        }
    };

    let truncated_str = if truncated || budget_truncated {
        "true"
    } else {
        "false"
    };
    println!("{}{}", header_base, truncated_str);

    for line in lines.iter().take(line_count) {
        println!("{}", line);
    }
}
