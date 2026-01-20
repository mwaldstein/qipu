use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, PathResult,
    SpanningTreeEntry, TreeLink, TreeNote, TreeOptions, TreeResult,
};
use crate::lib::graph::GraphProvider;
use crate::lib::index::Edge;
use crate::lib::store::Store;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// Wrapper for BinaryHeap to use as min-heap (ordered by accumulated cost)
#[derive(Debug, Clone)]
struct HeapEntry {
    node_id: String,
    accumulated_cost: HopCost,
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
            .reverse()
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

                // Calculate new accumulated cost for this edge
                let edge_cost = if opts.ignore_value {
                    // Unweighted: all edges cost 1.0
                    get_link_type_cost(edge.link_type.as_str())
                } else {
                    // Weighted: use get_edge_cost with target note's value
                    if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                        let value = meta.value.unwrap_or(50);
                        get_edge_cost(edge.link_type.as_str(), value)
                    } else {
                        get_link_type_cost(edge.link_type.as_str())
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

                // Add note metadata (use canonical ID)
                if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                    notes.push(TreeNote {
                        id: meta.id.clone(),
                        title: meta.title.clone(),
                        note_type: meta.note_type,
                        tags: meta.tags.clone(),
                        path: meta.path.clone(),
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

/// Perform BFS traversal from a root node
#[tracing::instrument(skip(provider, store, opts, compaction_ctx, equivalence_map), fields(root = %root, direction = ?opts.direction, max_hops = %opts.max_hops.value(), max_nodes, max_edges, max_fanout))]
pub fn bfs_traverse(
    provider: &dyn GraphProvider,
    store: &Store,
    root: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<TreeResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, HopCost)> = VecDeque::new();
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
    queue.push_back((root.to_string(), HopCost::from(0)));
    visited.insert(root.to_string());

    // Add root note
    if let Some(meta) = provider.get_metadata(root) {
        notes.push(TreeNote {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type,
            tags: meta.tags.clone(),
            path: meta.path.clone(),
        });
    }

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
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
            // If so, mark as truncated
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

                // Calculate new accumulated cost for this edge
                let edge_cost = get_link_type_cost(edge.link_type.as_str());
                let new_cost = accumulated_cost + edge_cost;

                // Add to spanning tree (first discovery, use canonical IDs)
                spanning_tree.push(SpanningTreeEntry {
                    from: current_id.clone(),
                    to: canonical_neighbor.clone(),
                    hop: new_cost.as_u32_for_display(),
                    link_type: edge.link_type.to_string(),
                });

                // Add note metadata (use canonical ID)
                if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                    notes.push(TreeNote {
                        id: meta.id.clone(),
                        title: meta.title.clone(),
                        note_type: meta.note_type,
                        tags: meta.tags.clone(),
                        path: meta.path.clone(),
                    });
                }

                // Queue for further expansion (use canonical ID)
                queue.push_back((canonical_neighbor, new_cost));
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
    // Check min_value filter for from and to notes before initializing
    let from_passes_filter = if let Some(meta) = provider.get_metadata(from) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    };

    if !from_passes_filter {
        return Ok(PathResult {
            from: from.to_string(),
            to: to.to_string(),
            direction: match opts.direction {
                Direction::Out => "out".to_string(),
                Direction::In => "in".to_string(),
                Direction::Both => "both".to_string(),
            },
            found: false,
            notes: vec![],
            links: vec![],
            path_length: 0,
        });
    }

    let to_passes_filter = if let Some(meta) = provider.get_metadata(to) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    };

    if !to_passes_filter {
        return Ok(PathResult {
            from: from.to_string(),
            to: to.to_string(),
            direction: match opts.direction {
                Direction::Out => "out".to_string(),
                Direction::In => "in".to_string(),
                Direction::Both => "both".to_string(),
            },
            found: false,
            notes: vec![],
            links: vec![],
            path_length: 0,
        });
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();

    // Track best-known cost to each node (for weighted mode)
    let mut best_costs: HashMap<String, HopCost> = HashMap::new();

    let mut found = false;

    if opts.ignore_value {
        // Unweighted BFS (use VecDeque)
        let mut queue: VecDeque<(String, HopCost)> = VecDeque::new();

        // Initialize
        queue.push_back((from.to_string(), HopCost::from(0)));
        visited.insert(from.to_string());
        best_costs.insert(from.to_string(), HopCost::from(0));

        while let Some((current_id, accumulated_cost)) = queue.pop_front() {
            // Check if we found the target
            if current_id == to {
                found = true;
                break;
            }

            // Don't expand beyond max_hops (use accumulated cost)
            if accumulated_cost.value() >= opts.max_hops.value() {
                continue;
            }

            // Get neighbors (gather edges from all compacted notes)
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
            // Inbound edges
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

                // Check min_value filter before processing neighbor
                let neighbor_passes_filter = if !visited.contains(&canonical_neighbor) {
                    if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                        let value = meta.value.unwrap_or(50);
                        opts.min_value.is_none_or(|min| value >= min)
                    } else {
                        false
                    }
                } else {
                    true
                };

                // Skip neighbor if it doesn't pass min_value filter and hasn't been visited
                if !neighbor_passes_filter {
                    continue;
                }

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
                    // Calculate new accumulated cost
                    let edge_cost = get_link_type_cost(edge.link_type.as_str());
                    let new_cost = accumulated_cost + edge_cost;
                    best_costs.insert(canonical_neighbor.clone(), new_cost);
                    queue.push_back((canonical_neighbor, new_cost));
                }
            }
        }
    } else {
        // Weighted Dijkstra (use BinaryHeap)
        let mut heap: BinaryHeap<Reverse<HeapEntry>> = BinaryHeap::new();

        // Initialize
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
            // Check if we found the target
            if current_id == to {
                found = true;
                break;
            }

            // Don't expand beyond max_hops (use accumulated cost)
            if accumulated_cost.value() >= opts.max_hops.value() {
                continue;
            }

            // Get neighbors (gather edges from all compacted notes)
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
            // Inbound edges
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

                // Check min_value filter before processing neighbor
                let neighbor_passes_filter = if !visited.contains(&canonical_neighbor) {
                    if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                        let value = meta.value.unwrap_or(50);
                        opts.min_value.is_none_or(|min| value >= min)
                    } else {
                        false
                    }
                } else {
                    true
                };

                // Skip neighbor if it doesn't pass min_value filter and hasn't been visited
                if !neighbor_passes_filter {
                    continue;
                }

                // Calculate new accumulated cost for this edge
                let edge_cost = if let Some(meta) = provider.get_metadata(&canonical_neighbor) {
                    let value = meta.value.unwrap_or(50);
                    get_edge_cost(edge.link_type.as_str(), value)
                } else {
                    get_link_type_cost(edge.link_type.as_str())
                };
                let new_cost = accumulated_cost + edge_cost;

                // Check if we found a better path to this node
                let should_visit = if let Some(&existing_cost) = best_costs.get(&canonical_neighbor)
                {
                    new_cost.value() < existing_cost.value() - 0.0001
                } else {
                    true
                };

                if should_visit {
                    if visited.contains(&canonical_neighbor) {
                        // Found a better path to an already-visited node
                    } else {
                        visited.insert(canonical_neighbor.clone());
                    }

                    // Update best cost and predecessor
                    best_costs.insert(canonical_neighbor.clone(), new_cost);
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

                    // Add to heap for further expansion
                    heap.push(Reverse(HeapEntry {
                        node_id: canonical_neighbor,
                        accumulated_cost: new_cost,
                    }));
                }
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
                provider.get_metadata(id).map(|meta| TreeNote {
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
mod tests {
    use super::*;
    use crate::lib::index::IndexBuilder;
    use crate::lib::store::Store;
    use tempfile::tempdir;

    /// Test HeapEntry comparison ordering
    #[test]
    fn test_heap_entry_ordering() {
        let entry1 = HeapEntry {
            node_id: "A".to_string(),
            accumulated_cost: HopCost::from(1),
        };
        let entry2 = HeapEntry {
            node_id: "B".to_string(),
            accumulated_cost: HopCost::from(2),
        };
        let entry3 = HeapEntry {
            node_id: "C".to_string(),
            accumulated_cost: HopCost::from(1),
        };

        // Lower cost should compare as greater (for min-heap)
        assert_eq!(entry1.cmp(&entry2), std::cmp::Ordering::Greater);
        assert_eq!(entry2.cmp(&entry1), std::cmp::Ordering::Less);

        // Equal costs with different node_ids
        assert_eq!(entry1.cmp(&entry3), std::cmp::Ordering::Equal);

        // PartialEq should work
        assert_eq!(entry1, entry1);
        assert_ne!(entry1, entry2);
    }

    /// Test that dijkstra_traverse works with ignore_value=true (unweighted)
    #[test]
    fn test_dijkstra_traverse_unweighted() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut root = store
            .create_note("Root Note", None, &["root".to_string()], None)
            .unwrap();
        root.frontmatter.value = Some(100);
        store.save_note(&mut root).unwrap();

        let mut mid = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        mid.frontmatter.value = Some(50);
        store.save_note(&mut mid).unwrap();

        let mut leaf = store
            .create_note("Leaf Note", None, &["leaf".to_string()], None)
            .unwrap();
        leaf.frontmatter.value = Some(0);
        store.save_note(&mut leaf).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

        assert_eq!(result.root, root.id());
        assert!(!result.truncated);
        assert_eq!(result.notes.len(), 1); // Only root (no links yet)
    }

    /// Test that dijkstra_traverse with ignore_value=false (weighted)
    #[test]
    fn test_dijkstra_traverse_weighted() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut root = store
            .create_note("Root Note", None, &["root".to_string()], None)
            .unwrap();
        root.frontmatter.value = Some(100);
        store.save_note(&mut root).unwrap();

        let mut mid = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        mid.frontmatter.value = Some(50);
        store.save_note(&mut mid).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: false,
            max_hops: HopCost::from(10),
            ..Default::default()
        };

        let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

        assert_eq!(result.root, root.id());
        assert!(!result.truncated);
        assert_eq!(result.notes.len(), 1);
    }

    /// Test that dijkstra_traverse respects min_value filter
    #[test]
    fn test_dijkstra_traverse_min_value_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut root = store
            .create_note("Root Note", None, &["root".to_string()], None)
            .unwrap();
        root.frontmatter.value = Some(100);
        store.save_note(&mut root).unwrap();

        let mut low = store
            .create_note("Low Value Note", None, &["low".to_string()], None)
            .unwrap();
        low.frontmatter.value = Some(30);
        store.save_note(&mut low).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            min_value: Some(50),
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

        assert_eq!(result.root, root.id());
        assert!(!result.truncated);
        assert_eq!(result.notes.len(), 1); // Only root (low-value note excluded)
    }

    /// Test that bfs_find_path works with ignore_value=true (unweighted)
    #[test]
    fn test_bfs_find_path_unweighted() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        from_note.frontmatter.value = Some(100);
        store.save_note(&mut from_note).unwrap();

        let mut mid_note = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        mid_note.frontmatter.value = Some(50);
        store.save_note(&mut mid_note).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        to_note.frontmatter.value = Some(0);
        store.save_note(&mut to_note).unwrap();

        // Create links: from -> mid -> to
        from_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: mid_note.id().to_string(),
            });
        store.save_note(&mut from_note).unwrap();

        mid_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: to_note.id().to_string(),
            });
        store.save_note(&mut mid_note).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = bfs_find_path(
            &index,
            &store,
            from_note.id(),
            to_note.id(),
            &opts,
            None,
            None,
        )
        .unwrap();

        assert!(result.found);
        assert_eq!(result.path_length, 2);
        assert_eq!(result.notes.len(), 3);
    }

    /// Test that bfs_find_path works with ignore_value=false (weighted)
    #[test]
    fn test_bfs_find_path_weighted() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        from_note.frontmatter.value = Some(100);
        store.save_note(&mut from_note).unwrap();

        let mut mid_note = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        mid_note.frontmatter.value = Some(50);
        store.save_note(&mut mid_note).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        to_note.frontmatter.value = Some(0);
        store.save_note(&mut to_note).unwrap();

        // Create links: from -> mid -> to
        from_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: mid_note.id().to_string(),
            });
        store.save_note(&mut from_note).unwrap();

        mid_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: to_note.id().to_string(),
            });
        store.save_note(&mut mid_note).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: false,
            max_hops: HopCost::from(10),
            ..Default::default()
        };

        let result = bfs_find_path(
            &index,
            &store,
            from_note.id(),
            to_note.id(),
            &opts,
            None,
            None,
        )
        .unwrap();

        assert!(result.found);
        assert_eq!(result.path_length, 2);
        assert_eq!(result.notes.len(), 3);
    }

    /// Test that bfs_find_path respects min_value filter
    #[test]
    fn test_bfs_find_path_min_value_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        from_note.frontmatter.value = Some(100);
        store.save_note(&mut from_note).unwrap();

        let mut low_mid = store
            .create_note("Low Mid Note", None, &["lowmid".to_string()], None)
            .unwrap();
        low_mid.frontmatter.value = Some(30);
        store.save_note(&mut low_mid).unwrap();

        let mut high_mid = store
            .create_note("High Mid Note", None, &["highmid".to_string()], None)
            .unwrap();
        high_mid.frontmatter.value = Some(80);
        store.save_note(&mut high_mid).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        to_note.frontmatter.value = Some(100);
        store.save_note(&mut to_note).unwrap();

        // Create links: from -> low_mid -> to and from -> high_mid -> to
        from_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: low_mid.id().to_string(),
            });
        store.save_note(&mut from_note).unwrap();

        low_mid.frontmatter.links.push(crate::lib::note::TypedLink {
            link_type: crate::lib::note::LinkType::from("supports"),
            id: to_note.id().to_string(),
        });
        store.save_note(&mut low_mid).unwrap();

        from_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: high_mid.id().to_string(),
            });
        store.save_note(&mut from_note).unwrap();

        high_mid
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: to_note.id().to_string(),
            });
        store.save_note(&mut high_mid).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            min_value: Some(50),
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = bfs_find_path(
            &index,
            &store,
            from_note.id(),
            to_note.id(),
            &opts,
            None,
            None,
        )
        .unwrap();

        // Should find the path through high_mid (excludes low_mid due to min_value filter)
        assert!(result.found);
        assert_eq!(result.path_length, 2);
        assert_eq!(result.notes.len(), 3);
    }
}
