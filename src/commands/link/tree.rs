//! Link tree command
use std::collections::{HashMap, HashSet};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Index, IndexBuilder};
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

use super::{resolve_note_id, TreeLink, TreeOptions, TreeResult};

/// Execute the link tree command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Resolve note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build the index
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

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

    // Canonicalize the root note ID
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    // Verify note exists (check canonical ID)
    if !index.contains(&canonical_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Perform BFS traversal with compaction context
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    let result = if tree_opts.ignore_value {
        crate::lib::graph::bfs_traverse(
            &index,
            store,
            &canonical_id,
            &tree_opts,
            compaction_ctx.as_ref(),
            equivalence_map.as_ref(),
        )?
    } else {
        crate::lib::graph::dijkstra_traverse(
            &index,
            store,
            &canonical_id,
            &tree_opts,
            compaction_ctx.as_ref(),
            equivalence_map.as_ref(),
        )?
    };

    // Output
    match cli.format {
        OutputFormat::Json => {
            output_tree_json(cli, &result, compaction_ctx.as_ref())?;
        }
        OutputFormat::Human => {
            output_tree_human(cli, &result, &index, compaction_ctx.as_ref());
        }
        OutputFormat::Records => {
            output_tree_records(&result, store, &opts, cli, compaction_ctx.as_ref());
        }
    }

    Ok(())
}

/// Output tree in JSON format
fn output_tree_json(
    cli: &Cli,
    result: &TreeResult,
    compaction_ctx: Option<&CompactionContext>,
) -> Result<()> {
    let mut json_result = serde_json::to_value(result)?;
    // Add compacted IDs if --with-compaction-ids is set
    if cli.with_compaction_ids {
        if let Some(ctx) = compaction_ctx {
            if let Some(notes) = json_result.get_mut("notes").and_then(|n| n.as_array_mut()) {
                for note in notes {
                    if let Some(id) = note.get("id").and_then(|i| i.as_str()) {
                        let compacts_count = ctx.get_compacts_count(id);
                        if compacts_count > 0 {
                            let depth = cli.compaction_depth.unwrap_or(1);
                            if let Some((ids, _truncated)) =
                                ctx.get_compacted_ids(id, depth, cli.compaction_max_nodes)
                            {
                                if let Some(obj_mut) = note.as_object_mut() {
                                    obj_mut.insert(
                                        "compacted_ids".to_string(),
                                        serde_json::json!(ids),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&json_result)?);
    Ok(())
}

/// Output tree in human-readable format
fn output_tree_human(
    cli: &Cli,
    result: &TreeResult,
    index: &Index,
    compaction_ctx: Option<&CompactionContext>,
) {
    if result.notes.is_empty() {
        if !cli.quiet {
            println!("No notes found");
        }
        return;
    }

    // Build tree structure for pretty printing
    // Use links (not spanning_tree) to include back-edges for (seen) rendering
    let mut children: HashMap<String, Vec<&TreeLink>> = HashMap::new();
    for link in &result.links {
        children.entry(link.from.clone()).or_default().push(link);
    }

    struct TreePrintConfig<'a> {
        prefix: &'a str,
        is_last: bool,
        cli: &'a Cli,
        compaction_ctx: Option<&'a CompactionContext>,
    }

    // Print tree recursively
    fn print_tree(
        id: &str,
        children: &HashMap<String, Vec<&TreeLink>>,
        index: &Index,
        visited: &HashSet<String>,
        config: &TreePrintConfig<'_>,
    ) {
        let title = index
            .get_metadata(id)
            .map(|m| m.title.as_str())
            .unwrap_or("(unknown)");

        let connector = if config.prefix.is_empty() {
            ""
        } else if config.is_last {
            "└── "
        } else {
            "├── "
        };

        println!("{}{}{} \"{}\"", config.prefix, connector, id, title);

        // Show compacted IDs if --with-compaction-ids is set
        if config.cli.with_compaction_ids {
            if let Some(ctx) = config.compaction_ctx {
                let compacts_count = ctx.get_compacts_count(id);
                if compacts_count > 0 {
                    let depth = config.cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) =
                        ctx.get_compacted_ids(id, depth, config.cli.compaction_max_nodes)
                    {
                        let ids_str = ids.join(", ");
                        let suffix = if truncated {
                            let max = config.cli.compaction_max_nodes.unwrap_or(ids.len());
                            format!(" (truncated, showing {} of {})", max, compacts_count)
                        } else {
                            String::new()
                        };
                        println!("{}  Compacted: {}{}", config.prefix, ids_str, suffix);
                    }
                }
            }
        }

        if let Some(kids) = children.get(id) {
            let new_prefix = if config.prefix.is_empty() {
                "".to_string()
            } else if config.is_last {
                format!("{}    ", config.prefix)
            } else {
                format!("{}│   ", config.prefix)
            };

            for (i, entry) in kids.iter().enumerate() {
                let is_last_child = i == kids.len() - 1;
                if visited.contains(&entry.to) {
                    // Already seen - mark but don't recurse
                    let connector = if is_last_child {
                        "└── "
                    } else {
                        "├── "
                    };
                    let child_title = index
                        .get_metadata(&entry.to)
                        .map(|m| m.title.as_str())
                        .unwrap_or("(unknown)");
                    println!(
                        "{}{}{} \"{}\" (seen)",
                        new_prefix, connector, entry.to, child_title
                    );
                } else {
                    let mut new_visited = visited.clone();
                    new_visited.insert(entry.to.clone());
                    let child_config = TreePrintConfig {
                        prefix: &new_prefix,
                        is_last: is_last_child,
                        cli: config.cli,
                        compaction_ctx: config.compaction_ctx,
                    };
                    print_tree(&entry.to, children, index, &new_visited, &child_config);
                }
            }
        }
    }

    let mut initial_visited = HashSet::new();
    initial_visited.insert(result.root.clone());
    let initial_config = TreePrintConfig {
        prefix: "",
        is_last: true,
        cli,
        compaction_ctx,
    };
    print_tree(
        &result.root,
        &children,
        index,
        &initial_visited,
        &initial_config,
    );

    if result.truncated {
        println!();
        println!(
            "[truncated: {}]",
            result
                .truncation_reason
                .as_deref()
                .unwrap_or("limit reached")
        );
    }
}

/// Output tree in records format
fn output_tree_records(
    result: &TreeResult,
    store: &Store,
    opts: &TreeOptions,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    for note in &result.notes {
        let tags_csv = if note.tags.is_empty() {
            "-".to_string()
        } else {
            note.tags.join(",")
        };
        lines.push(format!(
            "N {} {} \"{}\" tags={}",
            note.id,
            note.note_type,
            escape_quotes(&note.title),
            tags_csv
        ));

        if cli.with_compaction_ids {
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&note.id);
                if compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) =
                        ctx.get_compacted_ids(&note.id, depth, cli.compaction_max_nodes)
                    {
                        for id in &ids {
                            lines.push(format!("D compacted {} from={}", id, note.id));
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

        if let Ok(full_note) = store.get_note(&note.id) {
            let summary = full_note.summary();
            if !summary.is_empty() {
                let summary_text = summary.lines().next().unwrap_or("").trim();
                if !summary_text.is_empty() {
                    lines.push(format!("S {} {}", note.id, summary_text));
                }
            }
        }
    }

    for link in &result.links {
        lines.push(format!(
            "E {} {} {} {}",
            link.from, link.link_type, link.to, link.source
        ));
    }

    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.tree root={} direction={} max_hops={} truncated=",
        store.root().display(),
        result.root,
        result.direction,
        result.max_hops
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
            if budget.is_none_or(|max| used + line_len <= max) {
                used += line_len;
                count += 1;
            } else {
                return (true, count);
            }
        }

        (false, count)
    }

    let (budget_truncated, line_count, truncated) = if result.truncated {
        let (budget_flag, count) = select_lines(header_len_true, budget, &lines);
        (budget_flag, count, true)
    } else {
        let (budget_flag, count) = select_lines(header_len_false, budget, &lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, budget, &lines);
            (budget_flag, count, true)
        }
    };

    let truncated_value = if truncated || budget_truncated {
        "true"
    } else {
        "false"
    };
    println!("{}{}", header_base, truncated_value);

    for line in lines.iter().take(line_count) {
        println!("{}", line);
    }
}
