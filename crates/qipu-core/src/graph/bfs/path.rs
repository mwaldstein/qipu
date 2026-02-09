//! Path reconstruction utilities for graph traversal

use crate::graph::types::{TreeLink, TreeNote};
use crate::graph::GraphProvider;
use crate::index::Edge;
use std::collections::HashMap;

pub struct PredecessorInfo {
    pub canonical_pred: String,
    pub original_id: Option<String>,
    pub edge: Edge,
}

pub fn reconstruct_path(
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
