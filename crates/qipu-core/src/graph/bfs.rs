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
                let canonical_neighbor = processed.canonical_neighbor;
                let is_new = !visited.contains(&canonical_neighbor);
                if is_new {
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

fn reconstruct_path(
    from: &str,
    to: &str,
    predecessors: &HashMap<String, PredecessorInfo>,
    provider: &dyn GraphProvider,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> (Vec<TreeNote>, Vec<TreeLink>) {
    let mut path_nodes: Vec<(String, Option<String>)> = Vec::new();
    let mut path_links: Vec<TreeLink> = Vec::new();

    let mut current = to.to_string();
    let mut via: Option<String> = None;
    path_nodes.push((current.clone(), via.take()));

    while current != from {
        if let Some(pred_info) = predecessors.get(&current) {
            let link_type_str = pred_info.edge.link_type.to_string();
            let source_str = pred_info.edge.source.to_string();
            path_links.push(TreeLink {
                from: pred_info.edge.from.clone(),
                to: pred_info.edge.to.clone(),
                link_type: link_type_str,
                source: source_str,
                via: pred_info.original_id.clone(),
            });
            current = pred_info.canonical_pred.clone();
            via = pred_info.original_id.clone();
            path_nodes.push((current.clone(), via.take()));
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
                note_type: meta.note_type.clone(),
                tags: meta.tags.clone(),
                path: meta.path.clone(),
                via: via.clone(),
            })
        })
        .collect();

    let mut updated_links = path_links;
    if let (Some(equiv_map), Some(first_link)) = (equivalence_map, updated_links.first_mut()) {
        if let Some(equiv_ids) = equiv_map.get(from) {
            if equiv_ids.len() > 1 {
                if let Some(original_id) = equiv_ids.iter().find(|id| *id != from) {
                    first_link.via = Some(original_id.clone());
                }
            }
        }
    }

    (tree_notes, updated_links)
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
        reconstruct_path(from, to, &predecessors, provider, equivalence_map)
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
