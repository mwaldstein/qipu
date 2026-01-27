use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::graph::algos::dijkstra::HeapEntry;
use crate::lib::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, PathResult, TreeLink,
    TreeNote, TreeOptions,
};
use crate::lib::graph::GraphProvider;
use crate::lib::index::Edge;
use crate::lib::store::Store;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

struct PredecessorInfo {
    canonical_pred: String,
    original_id: Option<String>,
    edge: Edge,
}

fn check_min_value_filter(
    provider: &dyn GraphProvider,
    note_id: &str,
    min_value: Option<u8>,
) -> bool {
    if let Some(meta) = provider.get_metadata(note_id) {
        let value = meta.value.unwrap_or(50);
        min_value.is_none_or(|min| value >= min)
    } else {
        false
    }
}

fn get_note_value(provider: &dyn GraphProvider, note_id: &str) -> Option<u8> {
    provider.get_metadata(note_id).and_then(|meta| meta.value)
}

fn canonicalize_with_context(ctx: Option<&CompactionContext>, id: &str) -> Option<String> {
    if let Some(compaction_ctx) = ctx {
        compaction_ctx.canon(id).ok()
    } else {
        Some(id.to_string())
    }
}

fn collect_neighbors(
    provider: &dyn GraphProvider,
    store: &Store,
    current_id: &str,
    opts: &TreeOptions,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Vec<(String, Edge)> {
    let source_ids = equivalence_map
        .and_then(|map| map.get(current_id).cloned())
        .unwrap_or_else(|| vec![current_id.to_string()]);

    let mut neighbors = Vec::new();

    if opts.direction == Direction::Out || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in provider.get_outbound_edges(source_id) {
                if filter_edge(&edge, opts) {
                    neighbors.push((edge.to.clone(), edge));
                }
            }
        }
    }

    if opts.direction == Direction::In || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in provider.get_inbound_edges(source_id) {
                if opts.semantic_inversion {
                    let virtual_edge = edge.invert(store.config());
                    if filter_edge(&virtual_edge, opts) {
                        neighbors.push((virtual_edge.to.clone(), virtual_edge));
                    }
                } else if filter_edge(&edge, opts) {
                    neighbors.push((edge.from.clone(), edge));
                }
            }
        }
    }

    neighbors.sort_by(|a, b| {
        a.1.link_type
            .cmp(&b.1.link_type)
            .then_with(|| a.0.cmp(&b.0))
    });

    neighbors
}

struct ProcessedEdge {
    canonical_from: String,
    canonical_to: String,
    canonical_neighbor: String,
}

fn canonicalize_edge(
    edge: &Edge,
    neighbor_id: &str,
    compaction_ctx: Option<&CompactionContext>,
) -> Option<ProcessedEdge> {
    let canonical_from = canonicalize_with_context(compaction_ctx, &edge.from)?;
    let canonical_to = canonicalize_with_context(compaction_ctx, &edge.to)?;

    if canonical_from == canonical_to {
        return None;
    }

    let canonical_neighbor = canonicalize_with_context(compaction_ctx, neighbor_id)?;

    Some(ProcessedEdge {
        canonical_from,
        canonical_to,
        canonical_neighbor,
    })
}

fn check_can_visit(
    provider: &dyn GraphProvider,
    neighbor_id: &str,
    visited: &HashSet<String>,
    min_value: Option<u8>,
) -> bool {
    if visited.contains(neighbor_id) {
        return false;
    }
    check_min_value_filter(provider, neighbor_id, min_value)
}

fn bfs_search(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> (bool, HashMap<String, PredecessorInfo>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut predecessors: HashMap<String, PredecessorInfo> = HashMap::new();
    let mut queue: VecDeque<(String, HopCost)> = VecDeque::new();

    queue.push_back((from.to_string(), HopCost::from(0)));
    visited.insert(from.to_string());

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        if current_id == to {
            return (true, predecessors);
        }

        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        let neighbors = collect_neighbors(provider, store, &current_id, opts, equivalence_map);

        for (neighbor_id, edge) in neighbors {
            let Some(processed) = canonicalize_edge(&edge, &neighbor_id, compaction_ctx) else {
                continue;
            };

            if !check_can_visit(
                provider,
                &processed.canonical_neighbor,
                &visited,
                opts.min_value,
            ) {
                continue;
            }

            visited.insert(processed.canonical_neighbor.clone());
            let canonical_edge = Edge {
                from: processed.canonical_from,
                to: processed.canonical_to,
                link_type: edge.link_type.clone(),
                source: edge.source,
            };
            let original_id = if neighbor_id != processed.canonical_neighbor {
                Some(neighbor_id.clone())
            } else {
                None
            };
            predecessors.insert(
                processed.canonical_neighbor.clone(),
                PredecessorInfo {
                    canonical_pred: current_id.clone(),
                    original_id,
                    edge: canonical_edge,
                },
            );

            let edge_cost = get_link_type_cost(edge.link_type.as_str(), store.config());
            let new_cost = accumulated_cost + edge_cost;
            queue.push_back((processed.canonical_neighbor, new_cost));
        }
    }

    (false, predecessors)
}

fn check_dijkstra_can_visit(
    provider: &dyn GraphProvider,
    neighbor_id: &str,
    visited: &HashSet<String>,
    min_value: Option<u8>,
) -> bool {
    if visited.contains(neighbor_id) {
        return true;
    }
    if let Some(meta) = provider.get_metadata(neighbor_id) {
        let value = meta.value.unwrap_or(50);
        min_value.is_none_or(|min| value >= min)
    } else {
        false
    }
}

fn dijkstra_search(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> (bool, HashMap<String, PredecessorInfo>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut best_costs: HashMap<String, HopCost> = HashMap::new();
    let mut predecessors: HashMap<String, PredecessorInfo> = HashMap::new();
    let mut heap: BinaryHeap<Reverse<HeapEntry>> = BinaryHeap::new();

    heap.push(Reverse(HeapEntry {
        node_id: from.to_string(),
        accumulated_cost: HopCost::from(0),
    }));
    visited.insert(from.to_string());
    best_costs.insert(from.to_string(), HopCost::from(0));

    while let Some(Reverse(HeapEntry {
        node_id: current_id,
        accumulated_cost,
    })) = heap.pop()
    {
        if current_id == to {
            return (true, predecessors);
        }

        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        let neighbors = collect_neighbors(provider, store, &current_id, opts, equivalence_map);

        for (neighbor_id, edge) in neighbors {
            let Some(processed) = canonicalize_edge(&edge, &neighbor_id, compaction_ctx) else {
                continue;
            };

            if !check_dijkstra_can_visit(
                provider,
                &processed.canonical_neighbor,
                &visited,
                opts.min_value,
            ) {
                continue;
            }

            let edge_cost = match get_note_value(provider, &processed.canonical_neighbor) {
                Some(value) => get_edge_cost(edge.link_type.as_str(), value, store.config()),
                None => get_link_type_cost(edge.link_type.as_str(), store.config()),
            };
            let new_cost = accumulated_cost + edge_cost;

            let should_visit =
                if let Some(&existing_cost) = best_costs.get(&processed.canonical_neighbor) {
                    new_cost.value() < existing_cost.value() - 0.0001
                } else {
                    true
                };

            if should_visit {
                if !visited.contains(&processed.canonical_neighbor) {
                    visited.insert(processed.canonical_neighbor.clone());
                }

                best_costs.insert(processed.canonical_neighbor.clone(), new_cost);
                let canonical_edge = Edge {
                    from: processed.canonical_from,
                    to: processed.canonical_to,
                    link_type: edge.link_type.clone(),
                    source: edge.source,
                };
                let original_id = if neighbor_id != processed.canonical_neighbor {
                    Some(neighbor_id.clone())
                } else {
                    None
                };
                predecessors.insert(
                    processed.canonical_neighbor.clone(),
                    PredecessorInfo {
                        canonical_pred: current_id.clone(),
                        original_id,
                        edge: canonical_edge,
                    },
                );

                heap.push(Reverse(HeapEntry {
                    node_id: processed.canonical_neighbor,
                    accumulated_cost: new_cost,
                }));
            }
        }
    }

    (false, predecessors)
}

fn create_empty_path_result(from: &str, to: &str, direction: Direction) -> PathResult {
    PathResult {
        from: from.to_string(),
        to: to.to_string(),
        direction: match direction {
            Direction::Out => "out".to_string(),
            Direction::In => "in".to_string(),
            Direction::Both => "both".to_string(),
        },
        found: false,
        notes: vec![],
        links: vec![],
        path_length: 0,
    }
}

fn reconstruct_path(
    from: &str,
    to: &str,
    predecessors: &HashMap<String, PredecessorInfo>,
    provider: &dyn GraphProvider,
) -> (Vec<TreeNote>, Vec<TreeLink>) {
    let mut path_nodes: Vec<(String, Option<String>)> = Vec::new();
    let mut path_links: Vec<TreeLink> = Vec::new();

    let mut current = to.to_string();
    let mut via: Option<String> = None;
    path_nodes.push((current.clone(), via.clone()));

    while current != from {
        if let Some(pred_info) = predecessors.get(&current) {
            path_links.push(TreeLink {
                from: pred_info.edge.from.clone(),
                to: pred_info.edge.to.clone(),
                link_type: pred_info.edge.link_type.to_string(),
                source: pred_info.edge.source.to_string(),
            });
            current = pred_info.canonical_pred.clone();
            via = pred_info.original_id.clone();
            path_nodes.push((current.clone(), via.clone()));
        } else {
            break;
        }
    }

    path_nodes.reverse();
    path_links.reverse();

    let tree_notes: Vec<TreeNote> = path_nodes
        .iter()
        .filter_map(|(id, via)| {
            provider.get_metadata(id).map(|meta| TreeNote {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type,
                tags: meta.tags.clone(),
                path: meta.path.clone(),
                via: via.clone(),
            })
        })
        .collect();

    (tree_notes, path_links)
}

/// Find path between two nodes using BFS or Dijkstra
/// With `ignore_value=true`: unweighted BFS (all edges cost 1.0)
/// With `ignore_value=false`: weighted Dijkstra (cost based on note value)
#[tracing::instrument(skip(provider, store, opts, compaction_ctx, equivalence_map), fields(from = %from, to = %to, direction = ?opts.direction, max_hops = %opts.max_hops.value(), ignore_value = %opts.ignore_value))]
pub fn bfs_find_path(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<PathResult> {
    if !check_min_value_filter(provider, from, opts.min_value) {
        return Ok(create_empty_path_result(from, to, opts.direction));
    }

    if !check_min_value_filter(provider, to, opts.min_value) {
        return Ok(create_empty_path_result(from, to, opts.direction));
    }

    let (found, predecessors) = if opts.ignore_value {
        bfs_search(
            provider,
            store,
            from,
            to,
            opts,
            compaction_ctx,
            equivalence_map,
        )
    } else {
        dijkstra_search(
            provider,
            store,
            from,
            to,
            opts,
            compaction_ctx,
            equivalence_map,
        )
    };

    let (notes, links) = if found {
        reconstruct_path(from, to, &predecessors, provider)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(PathResult {
        from: from.to_string(),
        to: to.to_string(),
        direction: match opts.direction {
            Direction::Out => "out".to_string(),
            Direction::In => "in".to_string(),
            Direction::Both => "both".to_string(),
        },
        found,
        path_length: links.len(),
        notes,
        links,
    })
}

#[cfg(test)]
mod tests;
