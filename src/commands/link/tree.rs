//! Link tree command
use std::collections::{HashMap, HashSet, VecDeque};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Index, IndexBuilder};
use crate::lib::store::Store;

use super::{
    resolve_note_id, Direction, SpanningTreeEntry, TreeLink, TreeNote, TreeOptions, TreeResult,
};

/// Execute the link tree command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
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

    let result = crate::lib::graph::bfs_traverse(
        &index,
        store,
        &canonical_id,
        &tree_opts,
        compaction_ctx.as_ref(),
        equivalence_map.as_ref(),
    )?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            let mut json_result = serde_json::to_value(&result)?;
            // Add compacted IDs if --with-compaction-ids is set
            if cli.with_compaction_ids {
                if let Some(ref ctx) = compaction_ctx {
                    if let Some(notes) = json_result.get_mut("notes").and_then(|n| n.as_array_mut())
                    {
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

/// Perform BFS traversal from a root node with optional compaction resolution
fn bfs_traverse(
    cli: &Cli,
    index: &Index,
    store: &Store,
    root: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<TreeResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    let mut notes: Vec<TreeNote> = Vec::new();
    let mut links: Vec<TreeLink> = Vec::new();
    let mut spanning_tree: Vec<SpanningTreeEntry> = Vec::new();

    let mut truncated = false;
    let mut truncation_reason: Option<String> = None;

    // Initialize with root
    queue.push_back((root.to_string(), 0));
    visited.insert(root.to_string());

    // Add root note
    if let Some(meta) = index.get_metadata(root) {
        notes.push(TreeNote {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type,
            tags: meta.tags.clone(),
            path: meta.path.clone(),
        });
    }

    while let Some((current_id, hop)) = queue.pop_front() {
        // Check max_nodes limit
        if let Some(max) = opts.max_nodes {
            if visited.len() >= max {
                truncated = true;
                truncation_reason = Some("max_nodes".to_string());
                break;
            }
        }

        // Check max_edges limit
        if let Some(max) = opts.max_edges {
            if links.len() >= max {
                truncated = true;
                truncation_reason = Some("max_edges".to_string());
                break;
            }
        }

        // Don't expand beyond max_hops
        if hop >= opts.max_hops {
            continue;
        }

        // Get neighbors based on direction (gather edges from all compacted notes)
        let source_ids = equivalence_map
            .and_then(|map| map.get(&current_id).cloned())
            .unwrap_or_else(|| vec![current_id.clone()]);

        let mut neighbors = Vec::new();

        // Outbound edges
        if opts.direction == Direction::Out || opts.direction == Direction::Both {
            for source_id in &source_ids {
                for edge in index.get_outbound_edges(source_id) {
                    if super::filter_edge(edge, opts) {
                        neighbors.push((edge.to.clone(), edge.clone()));
                    }
                }
            }
        }

        // Inbound edges (Inversion point)
        if opts.direction == Direction::In || opts.direction == Direction::Both {
            for source_id in &source_ids {
                for edge in index.get_inbound_edges(source_id) {
                    if !cli.no_semantic_inversion {
                        // Virtual Inversion
                        let virtual_edge = edge.invert(store.config());
                        if super::filter_edge(&virtual_edge, opts) {
                            neighbors.push((virtual_edge.to.clone(), virtual_edge));
                        }
                    } else {
                        // Raw backlink
                        if super::filter_edge(edge, opts) {
                            neighbors.push((edge.from.clone(), edge.clone()));
                        }
                    }
                }
            }
        }

        // Sort for determinism
        neighbors.sort_by(|a, b| {
            a.1.link_type
                .cmp(&b.1.link_type)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Apply max_fanout
        let neighbors: Vec<_> = if let Some(max_fanout) = opts.max_fanout {
            if neighbors.len() > max_fanout {
                truncated = true;
                truncation_reason = Some("max_fanout".to_string());
            }
            neighbors.into_iter().take(max_fanout).collect()
        } else {
            neighbors
        };

        for (neighbor_id, edge) in neighbors {
            // Canonicalize edge endpoints if using compaction
            let canonical_from = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.from)?
            } else {
                edge.from.clone()
            };
            let canonical_to = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.to)?
            } else {
                edge.to.clone()
            };

            // Skip self-loops introduced by compaction contraction
            if canonical_from == canonical_to {
                continue;
            }

            // Canonicalize the neighbor ID
            let canonical_neighbor = if let Some(ctx) = compaction_ctx {
                ctx.canon(&neighbor_id)?
            } else {
                neighbor_id.clone()
            };

            // Check max_edges again before adding
            if let Some(max) = opts.max_edges {
                if links.len() >= max {
                    truncated = true;
                    truncation_reason = Some("max_edges".to_string());
                    break;
                }
            }

            // Add edge with canonical IDs
            links.push(TreeLink {
                from: canonical_from,
                to: canonical_to,
                link_type: edge.link_type.to_string(),
                source: edge.source.to_string(),
            });

            // Process neighbor if not visited (use canonical ID)
            if !visited.contains(&canonical_neighbor) {
                // Check max_nodes before adding
                if let Some(max) = opts.max_nodes {
                    if visited.len() >= max {
                        truncated = true;
                        truncation_reason = Some("max_nodes".to_string());
                        break;
                    }
                }

                visited.insert(canonical_neighbor.clone());

                // Add to spanning tree (first discovery, use canonical IDs)
                spanning_tree.push(SpanningTreeEntry {
                    from: current_id.clone(),
                    to: canonical_neighbor.clone(),
                    hop: hop + 1,
                });

                // Add note metadata (use canonical ID)
                if let Some(meta) = index.get_metadata(&canonical_neighbor) {
                    notes.push(TreeNote {
                        id: meta.id.clone(),
                        title: meta.title.clone(),
                        note_type: meta.note_type,
                        tags: meta.tags.clone(),
                        path: meta.path.clone(),
                    });
                }

                // Queue for further expansion (use canonical ID)
                queue.push_back((canonical_neighbor, hop + 1));
            }
        }
    }

    // Sort for determinism
    notes.sort_by(|a, b| a.id.cmp(&b.id));
    links.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
    spanning_tree.sort_by(|a, b| a.hop.cmp(&b.hop).then_with(|| a.to.cmp(&b.to)));

    Ok(TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            crate::commands::link::Direction::Out => "out".to_string(),
            crate::commands::link::Direction::In => "in".to_string(),
            crate::commands::link::Direction::Both => "both".to_string(),
        },
        max_hops: opts.max_hops,
        truncated,
        truncation_reason,
        notes,
        links,
        spanning_tree,
    })
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
    let mut children: HashMap<String, Vec<&SpanningTreeEntry>> = HashMap::new();
    for entry in &result.spanning_tree {
        children.entry(entry.from.clone()).or_default().push(entry);
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
            note.id, note.note_type, note.title, tags_csv
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
            if budget.map_or(true, |max| used + line_len <= max) {
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
