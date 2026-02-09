mod path;

use crate::compaction::CompactionContext;
use crate::error::Result;
use crate::graph::algos::dijkstra::HeapEntry;
use crate::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, PathResult, TreeLink,
    TreeNote, TreeOptions, DIRECTION_BOTH, DIRECTION_IN, DIRECTION_OUT,
};
use crate::graph::GraphProvider;
use crate::index::Edge;
use crate::store::Store;
use path::PredecessorInfo;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

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
    let source_ids: &[&str] = &match equivalence_map.and_then(|map| map.get(current_id)) {
        Some(ids) if !ids.is_empty() => {
            let mut v: Vec<&str> = Vec::with_capacity(ids.len());
            for id in ids {
                v.push(id.as_str());
            }
            v
        }
        _ => vec![current_id],
    };

    let mut neighbors = Vec::new();

    if opts.direction == Direction::Out || opts.direction == Direction::Both {
        for source_id in source_ids {
            for edge in provider.get_outbound_edges(source_id) {
                if filter_edge(edge, opts) {
                    let edge_clone = edge.clone();
                    neighbors.push((edge_clone.to.clone(), edge_clone));
                }
            }
        }
    }

    if opts.direction == Direction::In || opts.direction == Direction::Both {
        for source_id in source_ids {
            for edge in provider.get_inbound_edges(source_id) {
                if opts.semantic_inversion {
                    let virtual_edge = edge.invert(store.config());
                    if filter_edge(&virtual_edge, opts) {
                        let target = virtual_edge.to.clone();
                        neighbors.push((target, virtual_edge));
                    }
                } else if filter_edge(edge, opts) {
                    let edge_clone = edge.clone();
                    neighbors.push((edge_clone.from.clone(), edge_clone));
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

    let from_owned = from.to_string();
    queue.push_back((from_owned.clone(), HopCost::from(0)));
    visited.insert(from_owned);

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        if current_id == to {
            return (true, predecessors);
        }

        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        // Check max_nodes limit before expanding
        if let Some(max_nodes) = opts.max_nodes {
            if visited.len() >= max_nodes {
                break;
            }
        }

        let neighbors = collect_neighbors(provider, store, &current_id, opts, equivalence_map);

        // Apply max_fanout to limit neighbors processed
        let neighbors_to_process = if let Some(max_fanout) = opts.max_fanout {
            neighbors.into_iter().take(max_fanout).collect::<Vec<_>>()
        } else {
            neighbors
        };

        for (neighbor_id, edge) in neighbors_to_process {
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

            // Check max_nodes limit before adding new node
            if let Some(max_nodes) = opts.max_nodes {
                if visited.len() >= max_nodes {
                    break;
                }
            }

            let canonical_neighbor = processed.canonical_neighbor;
            let link_type_cloned = edge.link_type.clone();
            let canonical_edge = Edge {
                from: processed.canonical_from,
                to: processed.canonical_to,
                link_type: link_type_cloned.clone(),
                source: edge.source,
            };
            let original_id = if neighbor_id != canonical_neighbor {
                Some(neighbor_id)
            } else {
                None
            };
            visited.insert(canonical_neighbor.clone());
            predecessors.insert(
                canonical_neighbor.clone(),
                PredecessorInfo {
                    canonical_pred: current_id.clone(),
                    original_id,
                    edge: canonical_edge,
                },
            );

            let edge_cost = get_link_type_cost(link_type_cloned.as_str(), store.config());
            let new_cost = accumulated_cost + edge_cost;
            queue.push_back((canonical_neighbor, new_cost));
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

    let from_owned = from.to_string();
    visited.insert(from_owned.clone());
    best_costs.insert(from_owned.clone(), HopCost::from(0));
    heap.push(Reverse(HeapEntry {
        node_id: from_owned,
        accumulated_cost: HopCost::from(0),
    }));

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

        // Check max_nodes limit before expanding
        if let Some(max_nodes) = opts.max_nodes {
            if visited.len() >= max_nodes {
                break;
            }
        }

        let neighbors = collect_neighbors(provider, store, &current_id, opts, equivalence_map);

        // Apply max_fanout to limit neighbors processed
        let neighbors_to_process = if let Some(max_fanout) = opts.max_fanout {
            neighbors.into_iter().take(max_fanout).collect::<Vec<_>>()
        } else {
            neighbors
        };

        for (neighbor_id, edge) in neighbors_to_process {
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
                let canonical_neighbor = processed.canonical_neighbor;
                let is_new = !visited.contains(&canonical_neighbor);

                // Check max_nodes limit before adding new node
                if is_new {
                    if let Some(max_nodes) = opts.max_nodes {
                        if visited.len() >= max_nodes {
                            break;
                        }
                    }
                    visited.insert(canonical_neighbor.clone());
                }

                best_costs.insert(canonical_neighbor.clone(), new_cost);
                let link_type_cloned = edge.link_type.clone();
                let canonical_edge = Edge {
                    from: processed.canonical_from,
                    to: processed.canonical_to,
                    link_type: link_type_cloned,
                    source: edge.source,
                };
                let original_id = if neighbor_id != canonical_neighbor {
                    Some(neighbor_id)
                } else {
                    None
                };
                predecessors.insert(
                    canonical_neighbor.clone(),
                    PredecessorInfo {
                        canonical_pred: current_id.clone(),
                        original_id,
                        edge: canonical_edge,
                    },
                );

                heap.push(Reverse(HeapEntry {
                    node_id: canonical_neighbor,
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
            Direction::Out => DIRECTION_OUT.to_string(),
            Direction::In => DIRECTION_IN.to_string(),
            Direction::Both => DIRECTION_BOTH.to_string(),
        },
        found: false,
        notes: vec![],
        links: vec![],
        path_length: 0,
    }
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
        path::reconstruct_path(from, to, &predecessors, provider, equivalence_map)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(PathResult {
        from: from.to_string(),
        to: to.to_string(),
        direction: match opts.direction {
            Direction::Out => DIRECTION_OUT.to_string(),
            Direction::In => DIRECTION_IN.to_string(),
            Direction::Both => DIRECTION_BOTH.to_string(),
        },
        found,
        path_length: links.len(),
        notes,
        links,
    })
}

#[cfg(test)]
mod tests;
