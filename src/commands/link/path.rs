//! Link path command
use std::collections::{HashMap, HashSet, VecDeque};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Edge, Index, IndexBuilder};
use crate::lib::store::Store;

use super::{get_filtered_neighbors, resolve_note_id, PathResult, TreeEdge, TreeNode, TreeOptions};

/// Execute the link path command
pub fn execute(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    opts: TreeOptions,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

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
    let result = bfs_find_path(
        &index,
        &canonical_from,
        &canonical_to,
        &opts,
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
                    if let Some(nodes) = json_result.get_mut("nodes").and_then(|n| n.as_array_mut())
                    {
                        for node in nodes {
                            if let Some(id) = node.get("id").and_then(|i| i.as_str()) {
                                let compacts_count = ctx.get_compacts_count(id);
                                if compacts_count > 0 {
                                    let depth = cli.compaction_depth.unwrap_or(1);
                                    if let Some((ids, _truncated)) =
                                        ctx.get_compacted_ids(id, depth, cli.compaction_max_nodes)
                                    {
                                        if let Some(obj_mut) = node.as_object_mut() {
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
fn bfs_find_path(
    index: &Index,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<PathResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();

    // Initialize
    queue.push_back((from.to_string(), 0));
    visited.insert(from.to_string());

    let mut found = false;

    while let Some((current_id, hop)) = queue.pop_front() {
        // Check if we found the target
        if current_id == to {
            found = true;
            break;
        }

        // Don't expand beyond max_hops
        if hop >= opts.max_hops {
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
                queue.push_back((canonical_neighbor, hop + 1));
            }
        }
    }

    // Reconstruct path if found
    let (nodes, edges) = if found {
        let mut path_nodes: Vec<String> = Vec::new();
        let mut path_edges: Vec<TreeEdge> = Vec::new();

        // Backtrack from target to source
        let mut current = to.to_string();
        path_nodes.push(current.clone());

        while current != from {
            if let Some((pred, edge)) = predecessors.get(&current) {
                path_edges.push(TreeEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                    link_type: edge.link_type.clone(),
                    source: edge.source.to_string(),
                });
                current = pred.clone();
                path_nodes.push(current.clone());
            } else {
                break;
            }
        }

        path_nodes.reverse();
        path_edges.reverse();

        // Convert to TreeNodes
        let tree_nodes: Vec<TreeNode> = path_nodes
            .iter()
            .filter_map(|id| {
                index.get_metadata(id).map(|meta| TreeNode {
                    id: meta.id.clone(),
                    title: meta.title.clone(),
                    note_type: meta.note_type,
                    tags: meta.tags.clone(),
                    path: meta.path.clone(),
                })
            })
            .collect();

        (tree_nodes, path_edges)
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
        path_length: edges.len(),
        nodes,
        edges,
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
    for (i, node) in result.nodes.iter().enumerate() {
        if i > 0 {
            // Print edge info
            if let Some(edge) = result.edges.get(i - 1) {
                println!("  |");
                println!("  | [{}] ({})", edge.link_type, edge.source);
                println!("  v");
            }
        }
        println!("{} \"{}\"", node.id, node.title);

        // Show compacted IDs if --with-compaction-ids is set
        if cli.with_compaction_ids {
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&node.id);
                if compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) =
                        ctx.get_compacted_ids(&node.id, depth, cli.compaction_max_nodes)
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
    let mut budget_truncated = false;
    let budget = opts.max_chars;

    // Collect output lines with budget tracking (only if path was found)
    let mut node_lines = Vec::new();
    let mut edge_lines = Vec::new();
    let mut used_chars = 0;

    // Estimate header size
    let header_estimate = 250; // Conservative estimate
    used_chars += header_estimate;

    if result.found {
        // Generate node lines (including summaries)
        for node in &result.nodes {
            let tags_csv = if node.tags.is_empty() {
                "-".to_string()
            } else {
                node.tags.join(",")
            };
            let node_line = format!(
                "N {} {} \"{}\" tags={}",
                node.id, node.note_type, node.title, tags_csv
            );

            let mut summary_line = None;
            // Load note to get summary (per spec: prefer summaries over full bodies)
            if let Ok(note) = store.get_note(&node.id) {
                let summary = note.summary();
                if !summary.is_empty() {
                    // Truncate summary to single line
                    let summary_text = summary.lines().next().unwrap_or("").trim();
                    if !summary_text.is_empty() {
                        summary_line = Some(format!("S {} {}", node.id, summary_text));
                    }
                }
            }

            // Calculate compaction info for this node
            let mut compacted_lines = Vec::new();
            if cli.with_compaction_ids {
                if let Some(ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&node.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&node.id, depth, cli.compaction_max_nodes)
                        {
                            for id in &ids {
                                compacted_lines
                                    .push(format!("D compacted {} from={}", id, node.id));
                            }
                            if truncated {
                                compacted_lines.push(format!(
                                    "D compacted_truncated max={} total={}",
                                    cli.compaction_max_nodes.unwrap_or(ids.len()),
                                    compacts_count
                                ));
                            }
                        }
                    }
                }
            }

            // Check budget before adding (10% safety buffer)
            let node_size = node_line.len()
                + 1
                + summary_line.as_ref().map_or(0, |s| s.len() + 1)
                + compacted_lines.iter().map(|l| l.len() + 1).sum::<usize>();
            let node_size_with_buffer = node_size + (node_size / 10);

            if let Some(max) = budget {
                if used_chars + node_size_with_buffer > max {
                    budget_truncated = true;
                    break;
                }
            }

            node_lines.push((node_line, summary_line, compacted_lines));
            used_chars += node_size;
        }

        // Generate edge lines
        if !budget_truncated {
            for edge in &result.edges {
                let edge_line = format!(
                    "E {} {} {} {}",
                    edge.from, edge.link_type, edge.to, edge.source
                );

                let edge_size = edge_line.len() + 1;
                let edge_size_with_buffer = edge_size + (edge_size / 10);

                if let Some(max) = budget {
                    if used_chars + edge_size_with_buffer > max {
                        budget_truncated = true;
                        break;
                    }
                }

                edge_lines.push(edge_line);
                used_chars += edge_size;
            }
        }
    }

    // Generate header (budget truncation doesn't apply to path - paths are small)
    // But we include the logic for consistency
    let found_str = if result.found { "true" } else { "false" };
    let truncated_str = if budget_truncated { "true" } else { "false" };
    println!(
        "H qipu=1 records=1 store={} mode=link.path from={} to={} direction={} found={} length={} truncated={}",
        store.root().display(),
        result.from,
        result.to,
        result.direction,
        found_str,
        result.path_length,
        truncated_str
    );

    // Output collected lines
    for (node_line, summary_line, compacted_lines) in node_lines {
        println!("{}", node_line);
        for line in compacted_lines {
            println!("{}", line);
        }
        if let Some(s) = summary_line {
            println!("{}", s);
        }
    }

    for edge_line in edge_lines {
        println!("{}", edge_line);
    }
}
