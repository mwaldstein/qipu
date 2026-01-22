use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::graph::types::{
    filter_edge, get_link_type_cost, Direction, HopCost, SpanningTreeEntry, TreeLink, TreeNote,
    TreeOptions, TreeResult,
};
use crate::lib::graph::GraphProvider;
use crate::lib::store::Store;
use std::collections::{HashMap, HashSet, VecDeque};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::index::IndexBuilder;
    use crate::lib::store::Store;
    use tempfile::tempdir;

    /// Test that bfs_traverse works correctly
    #[test]
    fn test_bfs_traverse_basic() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let root = store
            .create_note("Root Note", None, &["root".to_string()], None)
            .unwrap();
        let mid = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        let leaf = store
            .create_note("Leaf Note", None, &["leaf".to_string()], None)
            .unwrap();

        // Create links: root -> mid -> leaf
        let mut root_note = store.get_note(root.id()).unwrap();
        root_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: mid.id().to_string(),
            });
        store.save_note(&mut root_note).unwrap();

        let mut mid_note = store.get_note(mid.id()).unwrap();
        mid_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: leaf.id().to_string(),
            });
        store.save_note(&mut mid_note).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = bfs_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

        assert_eq!(result.root, root.id());
        assert!(!result.truncated);
        assert_eq!(result.notes.len(), 3); // root + mid + leaf
    }
}
