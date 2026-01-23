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

fn bfs_search(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> (bool, HashMap<String, (String, Edge)>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();
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
            let Some(canonical_from) = canonicalize_with_context(compaction_ctx, &edge.from) else {
                continue;
            };
            let Some(canonical_to) = canonicalize_with_context(compaction_ctx, &edge.to) else {
                continue;
            };

            if canonical_from == canonical_to {
                continue;
            }

            let Some(canonical_neighbor) = canonicalize_with_context(compaction_ctx, &neighbor_id)
            else {
                continue;
            };

            if visited.contains(&canonical_neighbor) {
                continue;
            }

            if !check_min_value_filter(provider, &canonical_neighbor, opts.min_value) {
                continue;
            }

            visited.insert(canonical_neighbor.clone());
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

            let edge_cost = get_link_type_cost(edge.link_type.as_str(), store.config());
            let new_cost = accumulated_cost + edge_cost;
            queue.push_back((canonical_neighbor, new_cost));
        }
    }

    (false, predecessors)
}

fn dijkstra_search(
    provider: &dyn GraphProvider,
    store: &Store,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> (bool, HashMap<String, (String, Edge)>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut best_costs: HashMap<String, HopCost> = HashMap::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();
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
            let Some(canonical_from) = canonicalize_with_context(compaction_ctx, &edge.from) else {
                continue;
            };
            let Some(canonical_to) = canonicalize_with_context(compaction_ctx, &edge.to) else {
                continue;
            };

            if canonical_from == canonical_to {
                continue;
            }

            let Some(canonical_neighbor) = canonicalize_with_context(compaction_ctx, &neighbor_id)
            else {
                continue;
            };

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

            if !neighbor_passes_filter {
                continue;
            }

            let edge_cost = match get_note_value(provider, &canonical_neighbor) {
                Some(value) => get_edge_cost(edge.link_type.as_str(), value, store.config()),
                None => get_link_type_cost(edge.link_type.as_str(), store.config()),
            };
            let new_cost = accumulated_cost + edge_cost;

            let should_visit = if let Some(&existing_cost) = best_costs.get(&canonical_neighbor) {
                new_cost.value() < existing_cost.value() - 0.0001
            } else {
                true
            };

            if should_visit {
                if !visited.contains(&canonical_neighbor) {
                    visited.insert(canonical_neighbor.clone());
                }

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
    predecessors: &HashMap<String, (String, Edge)>,
    provider: &dyn GraphProvider,
) -> (Vec<TreeNote>, Vec<TreeLink>) {
    let mut path_nodes: Vec<String> = Vec::new();
    let mut path_links: Vec<TreeLink> = Vec::new();

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
