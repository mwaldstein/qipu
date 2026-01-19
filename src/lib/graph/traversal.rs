use super::types::*;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::types::NoteMetadata;
use crate::lib::index::{Edge, Index};
use crate::lib::store::Store;
use std::collections::{HashMap, HashSet, VecDeque};

/// Trait for providing graph adjacency and metadata
pub trait GraphProvider {
    fn get_outbound_edges(&self, id: &str) -> Vec<Edge>;
    fn get_inbound_edges(&self, id: &str) -> Vec<Edge>;
    fn get_metadata(&self, id: &str) -> Option<NoteMetadata>;
    #[allow(dead_code)]
    fn contains(&self, id: &str) -> bool;
}

impl GraphProvider for Index {
    fn get_outbound_edges(&self, id: &str) -> Vec<Edge> {
        self.get_outbound_edges(id).into_iter().cloned().collect()
    }

    fn get_inbound_edges(&self, id: &str) -> Vec<Edge> {
        self.get_inbound_edges(id).into_iter().cloned().collect()
    }

    fn get_metadata(&self, id: &str) -> Option<NoteMetadata> {
        self.get_metadata(id).cloned()
    }

    #[allow(dead_code)]
    fn contains(&self, id: &str) -> bool {
        self.contains(id)
    }
}

/// Perform BFS traversal from a root node
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

/// Find path between two nodes using BFS
pub fn bfs_find_path(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<PathResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, HopCost)> = VecDeque::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();

    // Initialize
    queue.push_back((from.to_string(), HopCost::from(0)));
    visited.insert(from.to_string());

    let mut found = false;

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
                    } else {
                        if filter_edge(&edge, opts) {
                            neighbors.push((edge.from.clone(), edge));
                        }
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
                queue.push_back((canonical_neighbor, new_cost));
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
