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

            // Sort for determinism
            neighbors.sort_by(|a, b| {
                a.1.link_type
                    .cmp(&b.1.link_type)
                    .then_with(|| a.0.cmp(&b.0))
            });

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
                    let edge_cost = get_link_type_cost(edge.link_type.as_str(), store.config());
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

            // Sort for determinism
            neighbors.sort_by(|a, b| {
                a.1.link_type
                    .cmp(&b.1.link_type)
                    .then_with(|| a.0.cmp(&b.0))
            });

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
                    get_edge_cost(edge.link_type.as_str(), value, store.config())
                } else {
                    get_link_type_cost(edge.link_type.as_str(), store.config())
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

        // Should find path through high_mid (passes filter), not low_mid (excluded)
        assert!(result.found);
        assert_eq!(result.path_length, 2);
        assert_eq!(result.notes.len(), 3);
    }

    /// Test that bfs_find_path returns not found when target unreachable
    #[test]
    fn test_bfs_find_path_not_found() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        let to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();

        // No links created - notes are disconnected

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

        assert!(!result.found);
        assert_eq!(result.path_length, 0);
        assert_eq!(result.notes.len(), 0);
    }

    /// Test that bfs_find_path handles from/to notes that fail min_value filter
    #[test]
    fn test_bfs_find_path_from_fails_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        from_note.frontmatter.value = Some(10);
        store.save_note(&mut from_note).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        to_note.frontmatter.value = Some(90);
        store.save_note(&mut to_note).unwrap();

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

        assert!(!result.found); // From note fails filter
    }

    /// Test that bfs_find_path handles to notes that fail min_value filter
    #[test]
    fn test_bfs_find_path_to_fails_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        from_note.frontmatter.value = Some(90);
        store.save_note(&mut from_note).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        to_note.frontmatter.value = Some(10);
        store.save_note(&mut to_note).unwrap();

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

        assert!(!result.found); // To note fails filter
    }

    /// Test that bfs_find_path respects max_hops limit
    #[test]
    fn test_bfs_find_path_max_hops() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut from_note = store
            .create_note("From Note", None, &["from".to_string()], None)
            .unwrap();
        store.save_note(&mut from_note).unwrap();

        let mut mid1_note = store
            .create_note("Mid1 Note", None, &["mid1".to_string()], None)
            .unwrap();
        store.save_note(&mut mid1_note).unwrap();

        let mut mid2_note = store
            .create_note("Mid2 Note", None, &["mid2".to_string()], None)
            .unwrap();
        store.save_note(&mut mid2_note).unwrap();

        let mut to_note = store
            .create_note("To Note", None, &["to".to_string()], None)
            .unwrap();
        store.save_note(&mut to_note).unwrap();

        // Create links: from -> mid1 -> mid2 -> to (3 hops)
        from_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: mid1_note.id().to_string(),
            });
        store.save_note(&mut from_note).unwrap();

        mid1_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: mid2_note.id().to_string(),
            });
        store.save_note(&mut mid1_note).unwrap();

        mid2_note
            .frontmatter
            .links
            .push(crate::lib::note::TypedLink {
                link_type: crate::lib::note::LinkType::from("supports"),
                id: to_note.id().to_string(),
            });
        store.save_note(&mut mid2_note).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            max_hops: HopCost::from(2), // Limit to 2 hops, path needs 3
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

        assert!(!result.found); // Should not find path within max_hops limit
    }
}
