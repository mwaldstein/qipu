//! Link path command
use std::collections::{HashMap, HashSet, VecDeque};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Edge, Index, IndexBuilder};
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

use super::{get_filtered_neighbors, resolve_note_id, PathResult, TreeLink, TreeNote, TreeOptions};

/// Execute the link path command
pub fn execute(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    opts: TreeOptions,
) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

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

    // Canonicalize the note IDs
    let canonical_from = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&from_resolved)?
    } else {
        from_resolved.clone()
    };
    let canonical_to = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&to_resolved)?
    } else {
        to_resolved.clone()
    };

    // Verify both notes exist (check canonical IDs)
    if !index.contains(&canonical_from) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_from.clone(),
        });
    }
    if !index.contains(&canonical_to) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_to.clone(),
        });
    }

    // Find path using BFS with compaction context
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    let result = crate::lib::graph::bfs_find_path(
        &index,
        store,
        &canonical_from,
        &canonical_to,
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
            output_path_human(cli, &result, compaction_ctx.as_ref());
        }
        OutputFormat::Records => {
            output_path_records(&result, store, &opts, cli, compaction_ctx.as_ref());
        }
    }

    Ok(())
}

/// Find path between two nodes using BFS with optional compaction resolution
#[allow(dead_code)]
fn bfs_find_path(
    index: &Index,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<PathResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, crate::lib::graph::HopCost)> = VecDeque::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();

    // Initialize
    queue.push_back((from.to_string(), crate::lib::graph::HopCost::from(0)));
    visited.insert(from.to_string());

    let mut found = false;

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        // Check if we found the target
        if current_id == to {
            found = true;
            break;
        }

        // Don't expand beyond max_hops
        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        // Get neighbors (gather edges from all compacted notes)
        let neighbors = get_filtered_neighbors(index, &current_id, opts, equivalence_map);

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

            if !visited.contains(&canonical_neighbor) {
                visited.insert(canonical_neighbor.clone());
                // Store canonical edge
                let canonical_edge = Edge {
                    from: canonical_from,
                    to: canonical_to,
                    link_type: edge.link_type.clone(),
                    source: edge.source,
                };
                predecessors.insert(
                    canonical_neighbor.clone(),
                    (current_id.clone(), canonical_edge),
                );
                let edge_cost = crate::lib::graph::get_link_type_cost(edge.link_type.as_str());
                queue.push_back((canonical_neighbor, accumulated_cost + edge_cost));
            }
        }
    }

    // Reconstruct path if found
    let (notes, links) = if found {
        let mut path_nodes: Vec<String> = Vec::new();
        let mut path_links: Vec<TreeLink> = Vec::new();

        // Backtrack from target to source
        let mut current = to.to_string();
        path_nodes.push(current.clone());

        while current != from {
            if let Some((pred, edge)) = predecessors.get(&current) {
                path_links.push(TreeLink {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                    link_type: edge.link_type.to_string(),
                    source: edge.source.to_string(),
                });
                current = pred.clone();
                path_nodes.push(current.clone());
            } else {
                break;
            }
        }

        path_nodes.reverse();
        path_links.reverse();

        // Convert to TreeNotes
        let tree_notes: Vec<TreeNote> = path_nodes
            .iter()
            .filter_map(|id| {
                index.get_metadata(id).map(|meta| TreeNote {
                    id: meta.id.clone(),
                    title: meta.title.clone(),
                    note_type: meta.note_type,
                    tags: meta.tags.clone(),
                    path: meta.path.clone(),
                })
            })
            .collect();

        (tree_notes, path_links)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(PathResult {
        from: from.to_string(),
        to: to.to_string(),
        direction: match opts.direction {
            crate::commands::link::Direction::Out => "out".to_string(),
            crate::commands::link::Direction::In => "in".to_string(),
            crate::commands::link::Direction::Both => "both".to_string(),
        },
        found,
        path_length: links.len(),
        notes,
        links,
    })
}

/// Output path in human-readable format
fn output_path_human(cli: &Cli, result: &PathResult, compaction_ctx: Option<&CompactionContext>) {
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
        println!("{} \"{}\"", note.id, note.title);

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
    }

    println!();
    println!("Path length: {} hop(s)", result.path_length);
}

/// Output path in records format
fn output_path_records(
    result: &PathResult,
    store: &Store,
    opts: &TreeOptions,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    if result.found {
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
    }

    let found_str = if result.found { "true" } else { "false" };
    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.path from={} to={} direction={} found={} length={} truncated=",
        store.root().display(),
        result.from,
        result.to,
        result.direction,
        found_str,
        result.path_length
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

    let (budget_truncated, line_count, truncated) = if result.found {
        let (budget_flag, count) = select_lines(header_len_false, budget, &lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, budget, &lines);
            (budget_flag, count, true)
        }
    } else {
        let (budget_flag, count) = select_lines(header_len_false, budget, &lines);
        if !budget_flag {
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
