use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, SpanningTreeEntry,
    TreeLink, TreeNote, TreeOptions, TreeResult,
};
use crate::lib::graph::GraphProvider;
use crate::lib::store::Store;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Wrapper for BinaryHeap to use as min-heap (ordered by accumulated cost)
#[derive(Debug, Clone)]
pub struct HeapEntry {
    pub node_id: String,
    pub accumulated_cost: HopCost,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.accumulated_cost.value() == other.accumulated_cost.value()
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.accumulated_cost
            .value()
            .partial_cmp(&other.accumulated_cost.value())
            .unwrap()
    }
}

/// Perform Dijkstra traversal from a root node (weighted by note value)
/// Default behavior: weighted traversal using `get_edge_cost()` which
/// applies penalties for low-value notes (value 0-100).
/// With `--ignore-value`: unweighted BFS (all edges cost 1.0).
#[tracing::instrument(skip(provider, store, opts, compaction_ctx, equivalence_map), fields(root = %root, direction = ?opts.direction, max_hops = %opts.max_hops.value(), max_nodes, max_edges, max_fanout, ignore_value = %opts.ignore_value))]
pub fn dijkstra_traverse(
    provider: &dyn GraphProvider,
    store: &Store,
    root: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<TreeResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut heap: BinaryHeap<Reverse<HeapEntry>> = BinaryHeap::new();
    let mut notes: Vec<TreeNote> = Vec::new();
    let mut links: Vec<TreeLink> = Vec::new();
    let mut spanning_tree: Vec<SpanningTreeEntry> = Vec::new();

    let mut truncated = false;
    let mut truncation_reason: Option<String> = None;

    // Check min_value filter for root note before initializing
    let root_passes_filter = if let Some(meta) = provider.get_metadata(root) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    };

    if !root_passes_filter {
        return Ok(TreeResult {
            root: root.to_string(),
            direction: match opts.direction {
                Direction::Out => "out".to_string(),
                Direction::In => "in".to_string(),
                Direction::Both => "both".to_string(),
            },
            max_hops: opts.max_hops.as_u32_for_display(),
            truncated: false,
            truncation_reason: Some("min_value filter excluded root".to_string()),
            notes: vec![],
            links: vec![],
            spanning_tree: vec![],
        });
    }

    // Initialize with root
    heap.push(Reverse(HeapEntry {
        node_id: root.to_string(),
        accumulated_cost: HopCost::from(0),
    }));
    visited.insert(root.to_string());

    // Add root note
    if let Some(meta) = provider.get_metadata(root) {
        notes.push(TreeNote {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type,
            tags: meta.tags.clone(),
            path: meta.path.clone(),
            via: None,
        });
    }

    while let Some(Reverse(HeapEntry {
        node_id: current_id,
        accumulated_cost,
    })) = heap.pop()
    {
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

        // Don't expand beyond max_hops (use accumulated cost)
        if accumulated_cost.value() >= opts.max_hops.value() {
            // Check if there are any neighbors that would have been expanded
            let source_ids = equivalence_map
                .and_then(|map| map.get(&current_id).cloned())
                .unwrap_or_else(|| vec![current_id.clone()]);

            let has_unexpanded_neighbors =
                if opts.direction == Direction::Out || opts.direction == Direction::Both {
                    source_ids.iter().any(|id| {
                        provider
                            .get_outbound_edges(id)
                            .iter()
                            .any(|e| filter_edge(e, opts))
                    })
                } else {
                    false
                } || if opts.direction == Direction::In || opts.direction == Direction::Both {
                    source_ids.iter().any(|id| {
                        provider
                            .get_inbound_edges(id)
                            .iter()
                            .any(|e| filter_edge(e, opts))
                    })
                } else {
                    false
                };

            if has_unexpanded_neighbors {
                truncated = true;
                if truncation_reason.is_none() {
                    truncation_reason = Some("max_hops".to_string());
                }
            }

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
                for edge in provider.get_outbound_edges(source_id) {
                    if filter_edge(&edge, opts) {
                        neighbors.push((edge.to.clone(), edge));
                    }
                }
            }
        }

        // Inbound edges (Inversion point)
        if opts.direction == Direction::In || opts.direction == Direction::Both {
            for source_id in &source_ids {
                for edge in provider.get_inbound_edges(source_id) {
                    if opts.semantic_inversion {
                        // Virtual Inversion
                        let virtual_edge = edge.invert(store.config());
                        if filter_edge(&virtual_edge, opts) {
                            neighbors.push((virtual_edge.to.clone(), virtual_edge));
                        }
                    } else {
                        // Raw backlink
                        if filter_edge(&edge, opts) {
                            neighbors.push((edge.from.clone(), edge));
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

            // Check min_value filter before adding link or processing neighbor
            let neighbor_passes_filter = if !visited.contains(&canonical_neighbor) {
                if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                    let value = meta.value.unwrap_or(50);
                    opts.min_value.is_none_or(|min| value >= min)
                } else {
                    false
                }
            } else {
                true // Already visited, include the link
            };

            // Skip neighbor if it doesn't pass min_value filter and hasn't been visited
            if !neighbor_passes_filter {
                continue;
            }

            // Check max_edges again before adding
            if let Some(max) = opts.max_edges {
                if links.len() >= max {
                    truncated = true;
                    truncation_reason = Some("max_edges".to_string());
                    break;
                }
            }

            // Track via if neighbor was canonicalized
            let via = if neighbor_id != canonical_neighbor {
                Some(neighbor_id.clone())
            } else {
                None
            };

            // Add edge with canonical IDs and via annotation
            links.push(TreeLink {
                from: canonical_from,
                to: canonical_to,
                link_type: edge.link_type.to_string(),
                source: edge.source.to_string(),
                via: via.clone(),
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

                // Calculate new accumulated cost for this edge
                let edge_cost = if opts.ignore_value {
                    // Unweighted: all edges cost 1.0
                    get_link_type_cost(edge.link_type.as_str(), store.config())
                } else {
                    // Weighted: use get_edge_cost with target note's value
                    if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                        let value = meta.value.unwrap_or(50);
                        get_edge_cost(edge.link_type.as_str(), value, store.config())
                    } else {
                        get_link_type_cost(edge.link_type.as_str(), store.config())
                    }
                };
                let new_cost = accumulated_cost + edge_cost;

                // Add to spanning tree (first discovery, use canonical IDs)
                spanning_tree.push(SpanningTreeEntry {
                    from: current_id.clone(),
                    to: canonical_neighbor.clone(),
                    hop: new_cost.as_u32_for_display(),
                    link_type: edge.link_type.to_string(),
                });

                // Add note metadata (use canonical ID, track via if canonicalized)
                if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                    notes.push(TreeNote {
                        id: meta.id.clone(),
                        title: meta.title.clone(),
                        note_type: meta.note_type,
                        tags: meta.tags.clone(),
                        path: meta.path.clone(),
                        via,
                    });
                }

                // Add to heap for further expansion (use canonical ID)
                heap.push(Reverse(HeapEntry {
                    node_id: canonical_neighbor,
                    accumulated_cost: new_cost,
                }));
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
    spanning_tree.sort_by(|a, b| {
        a.hop
            .cmp(&b.hop)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });

    Ok(TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            Direction::Out => "out".to_string(),
            Direction::In => "in".to_string(),
            Direction::Both => "both".to_string(),
        },
        max_hops: opts.max_hops.as_u32_for_display(),
        truncated,
        truncation_reason,
        notes,
        links,
        spanning_tree,
    })
}

#[cfg(test)]
mod tests;
