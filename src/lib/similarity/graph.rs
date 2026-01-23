use crate::lib::index::types::Index;
use crate::lib::similarity::SimilarityResult;
use std::collections::HashMap;

/// Find notes within 2 hops in the link graph
///
/// This function performs a 2-hop graph traversal starting from the given note:
/// 1. First, collect all 1-hop neighbors (directly linked notes)
/// 2. Then, collect all 2-hop neighbors (notes linked to 1-hop neighbors)
/// 3. Score each 2-hop note by the number of different 2-hop paths to it
/// 4. Return results sorted by score (descending), limited to `limit` results
///
/// The 2-hop scoring helps identify notes that are related through multiple
/// intermediate nodes, which may indicate stronger indirect connections.
pub fn find_by_2hop_neighborhood(
    index: &Index,
    note_id: &str,
    limit: usize,
) -> Vec<SimilarityResult> {
    let mut results = Vec::new();
    let mut neighbor_counts: HashMap<String, usize> = HashMap::new();

    let outbound = index.get_outbound_edges(note_id);
    let inbound = index.get_inbound_edges(note_id);

    let mut one_hop = std::collections::HashSet::new();
    for edge in outbound {
        one_hop.insert(edge.to.clone());
    }
    for edge in inbound {
        one_hop.insert(edge.from.clone());
    }

    for neighbor_id in &one_hop {
        let outbound = index.get_outbound_edges(neighbor_id);
        let inbound = index.get_inbound_edges(neighbor_id);

        for edge in outbound {
            if edge.to != note_id && !one_hop.contains(&edge.to) {
                *neighbor_counts.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
        for edge in inbound {
            if edge.from != note_id && !one_hop.contains(&edge.from) {
                *neighbor_counts.entry(edge.from.clone()).or_insert(0) += 1;
            }
        }
    }

    for (id, count) in neighbor_counts {
        results.push(SimilarityResult {
            id,
            score: count as f64,
        });
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::index::types::{LinkSource, NoteMetadata};
    use crate::lib::note::{LinkType, NoteType};

    fn create_test_index() -> Index {
        let mut index = Index::new();

        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-1".to_string(),
            to: "qp-2".to_string(),
            link_type: LinkType::from("related"),
            source: LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-2".to_string(),
            to: "qp-3".to_string(),
            link_type: LinkType::from("related"),
            source: LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-1".to_string(),
            to: "qp-4".to_string(),
            link_type: LinkType::from("related"),
            source: LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-4".to_string(),
            to: "qp-3".to_string(),
            link_type: LinkType::from("related"),
            source: LinkSource::Inline,
        });

        for id in ["qp-1", "qp-2", "qp-3", "qp-4", "qp-5"] {
            index.metadata.insert(
                id.to_string(),
                NoteMetadata {
                    id: id.to_string(),
                    title: format!("Note {}", id),
                    note_type: NoteType::Permanent,
                    tags: vec![],
                    path: format!("{}.md", id),
                    created: None,
                    updated: None,
                    value: None,
                },
            );
        }

        index
    }

    #[test]
    fn test_2hop_neighborhood_basic() {
        let index = create_test_index();
        let results = find_by_2hop_neighborhood(&index, "qp-1", 100);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "qp-3");
        assert_eq!(results[0].score, 2.0);
    }

    #[test]
    fn test_2hop_excludes_1hop_and_isolated() {
        let index = create_test_index();
        let results = find_by_2hop_neighborhood(&index, "qp-1", 100);

        assert!(!results.iter().any(|r| r.id == "qp-1"));
        assert!(!results.iter().any(|r| r.id == "qp-2"));
        assert!(!results.iter().any(|r| r.id == "qp-4"));
        assert!(!results.iter().any(|r| r.id == "qp-5"));
    }

    #[test]
    fn test_2hop_limit() {
        let index = create_test_index();
        let results = find_by_2hop_neighborhood(&index, "qp-1", 1);

        assert_eq!(results.len(), 1);
    }
}
