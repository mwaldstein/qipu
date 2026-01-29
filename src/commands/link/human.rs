use super::LinkEntry;
use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::graph::types::SpanningTreeEntry;
use qipu_core::graph::{PathResult, TreeResult};
use qipu_core::index::Index;
use qipu_core::note::Note;
use std::collections::{HashMap, HashSet};

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    entries: &[LinkEntry],
    display_id: &str,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
) {
    if entries.is_empty() {
        if !cli.quiet {
            println!("No links found for {}", display_id);
        }
    } else {
        for entry in entries {
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

            // Build compaction annotations for digest nodes
            // Per spec (specs/compaction.md lines 113-122)
            let mut annotations = String::new();
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&entry.id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    // Calculate compaction percentage if we have note data
                    if let Some(map) = note_map {
                        if let Some(note) = map.get(entry.id.as_str()) {
                            if let Some(pct) = ctx.get_compaction_pct(note, map) {
                                annotations.push_str(&format!(" compaction={:.0}%", pct));
                            }
                        }
                    }
                }
            }

            println!(
                "{} {}{} [{}] ({})",
                dir_arrow, entry.id, title_part, entry.link_type, entry.source
            );

            if !annotations.is_empty() {
                println!("  {}", annotations.trim_start());
            }

            // Show compacted IDs if --with-compaction-ids is set
            if cli.with_compaction_ids {
                if let Some(ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&entry.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&entry.id, depth, cli.compaction_max_nodes)
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
}

/// Output path in human-readable format
pub fn output_path_human(
    cli: &Cli,
    result: &PathResult,
    _store: &qipu_core::store::Store,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    all_notes: &[qipu_core::note::Note],
) {
    if !result.found {
        if !cli.quiet {
            println!("No path found from {} to {}", result.from, result.to);
        }
        return;
    }

    // Print path: node -> node -> node
    for (i, note) in result.notes.iter().enumerate() {
        if i > 0 {
            // Print edge info
            if let Some(link) = result.links.get(i - 1) {
                println!("  |");
                println!("  | [{}] ({})", link.link_type, link.source);
                println!("  v");
            }
        }

        // Build compaction annotations for digest nodes
        // Per spec (specs/compaction.md lines 113-122)
        let mut annotations = String::new();
        if let Some(ctx) = compaction_ctx {
            let compacts_count = ctx.get_compacts_count(&note.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                // Calculate compaction percentage if we have note data
                if let Some(map) = note_map {
                    if let Some(full_note) = map.get(note.id.as_str()) {
                        if let Some(pct) = ctx.get_compaction_pct(full_note, map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }
                }
            }
        }

        println!("{} \"{}\"", note.id, note.title);

        if !annotations.is_empty() {
            println!("  {}", annotations.trim_start());
        }

        // Show compacted IDs if --with-compaction-ids is set
        if cli.with_compaction_ids {
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&note.id);
                if compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) =
                        ctx.get_compacted_ids(&note.id, depth, cli.compaction_max_nodes)
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

        // Expand compaction: include full compacted note content
        if cli.expand_compaction {
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&note.id);
                if compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((compacted_notes, _truncated)) = ctx.get_compacted_notes_expanded(
                        &note.id,
                        depth,
                        cli.compaction_max_nodes,
                        all_notes,
                    ) {
                        println!("  Compacted Notes:");
                        for compacted_note in compacted_notes {
                            println!(
                                "    #### {} ({})",
                                compacted_note.title(),
                                compacted_note.id()
                            );
                            println!("    Type: {}", compacted_note.note_type());
                            if !compacted_note.frontmatter.tags.is_empty() {
                                println!(
                                    "    Tags: {}",
                                    compacted_note.frontmatter.tags.join(", ")
                                );
                            }
                            println!(
                                "    {}",
                                compacted_note
                                    .body
                                    .lines()
                                    .take(3)
                                    .collect::<Vec<_>>()
                                    .join("\n    ")
                            );
                            println!();
                        }
                    }
                }
            }
        }
    }

    println!();
    println!("Path length: {} hop(s)", result.path_length);
}

/// Output tree in human-readable format
pub fn output_tree_human(
    cli: &Cli,
    result: &TreeResult,
    index: &Index,
    _store: &qipu_core::store::Store,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    all_notes: &[Note],
) {
    if result.notes.is_empty() {
        if !cli.quiet {
            println!("No notes found");
        }
        return;
    }

    let mut children: HashMap<String, Vec<&SpanningTreeEntry>> = HashMap::new();
    for entry in &result.spanning_tree {
        children.entry(entry.from.clone()).or_default().push(entry);
    }

    struct TreePrintConfig<'a> {
        prefix: &'a str,
        is_last: bool,
        cli: &'a Cli,
        compaction_ctx: Option<&'a CompactionContext>,
        note_map: Option<&'a HashMap<&'a str, &'a Note>>,
        all_notes: &'a [Note],
    }

    fn print_tree(
        id: &str,
        children: &HashMap<String, Vec<&SpanningTreeEntry>>,
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

        let mut annotations = String::new();
        if let Some(ctx) = config.compaction_ctx {
            let compacts_count = ctx.get_compacts_count(id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                if let Some(map) = config.note_map {
                    if let Some(note) = map.get(id) {
                        if let Some(pct) = ctx.get_compaction_pct(note, map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }
                }
            }
        }

        println!("{}{}{} \"{}\"", config.prefix, connector, id, title);

        if !annotations.is_empty() {
            println!("{}  {}", config.prefix, annotations.trim_start());
        }

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

        if config.cli.expand_compaction {
            if let Some(ctx) = config.compaction_ctx {
                let compacts_count = ctx.get_compacts_count(id);
                if compacts_count > 0 {
                    let depth = config.cli.compaction_depth.unwrap_or(1);
                    if let Some((compacted_notes, _truncated)) = ctx.get_compacted_notes_expanded(
                        id,
                        depth,
                        config.cli.compaction_max_nodes,
                        config.all_notes,
                    ) {
                        println!("{}  Compacted Notes:", config.prefix);
                        for compacted_note in compacted_notes {
                            println!(
                                "{}    #### {} ({})",
                                config.prefix,
                                compacted_note.title(),
                                compacted_note.id()
                            );
                            println!("{}    Type: {}", config.prefix, compacted_note.note_type());
                            if !compacted_note.frontmatter.tags.is_empty() {
                                println!(
                                    "{}    Tags: {}",
                                    config.prefix,
                                    compacted_note.frontmatter.tags.join(", ")
                                );
                            }
                            println!(
                                "{}    {}",
                                config.prefix,
                                compacted_note
                                    .body
                                    .lines()
                                    .take(3)
                                    .collect::<Vec<_>>()
                                    .join("\n{}    ")
                            );
                            println!();
                        }
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
                let entry: &&SpanningTreeEntry = entry;
                let is_last_child = i == kids.len() - 1;
                if visited.contains(&entry.to) {
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
                        note_map: config.note_map,
                        all_notes: config.all_notes,
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
        note_map,
        all_notes,
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
