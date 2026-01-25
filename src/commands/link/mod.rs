//! Link management commands for qipu
//!
//! Per spec (specs/cli-interface.md, specs/graph-traversal.md):
//! - `qipu link list <id>` - list links for a note
//! - `qipu link add <from> <to> --type <t>` - add typed link
//! - `qipu link remove <from> <to> --type <t>` - remove typed link
//! - `qipu link tree <id>` - traversal tree from note
//! - `qipu link path <from> <to>` - find path between notes

pub mod add;
pub mod human;
pub mod json;
pub mod list;
pub mod path;
pub mod records;
pub mod remove;
pub mod tree;

pub use crate::lib::graph::{Direction, TreeOptions};

use std::collections::HashMap;

use crate::lib::error::Result;
use crate::lib::graph::TreeResult;
use crate::lib::index::{Edge, Index, LinkSource};
use crate::lib::store::Store;

use serde::Serialize;

/// Link entry for output
#[derive(Debug, Clone, Serialize)]
pub struct LinkEntry {
    /// Direction relative to the queried note
    pub direction: String,
    /// The other note's ID
    pub id: String,
    /// The other note's title (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Link type (related, derived-from, supports, contradicts, part-of)
    #[serde(rename = "type")]
    pub link_type: String,
    /// Link source (typed or inline)
    pub source: String,
    /// Via annotation - the original note ID before canonicalization (if different)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

/// Resolve a note ID or path to a canonical note ID
pub fn resolve_note_id(store: &Store, id_or_path: &str) -> Result<String> {
    // If it looks like an ID (starts with qp-), try to use it directly
    if id_or_path.starts_with("qp-") {
        // First check if it's a valid ID that exists
        if store.note_exists(id_or_path) {
            return Ok(id_or_path.to_string());
        }

        // If not, try parsing as a filename like qp-xxxx-slug.md
        let id = id_or_path
            .trim_end_matches(".md")
            .split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");
        return Ok(id);
    }

    // Otherwise, try to find a note by path
    let notes = store.list_notes()?;
    for note in notes {
        if let Some(path) = &note.path {
            let path_str = path.display().to_string();
            if path_str.contains(id_or_path) || path_str.ends_with(id_or_path) {
                return Ok(note.id().to_string());
            }
        }
    }

    Err(crate::lib::error::QipuError::NoteNotFound {
        id: id_or_path.to_string(),
    })
}

/// Filter and convert an outbound edge to a LinkEntry
pub fn filter_and_convert(
    edge: &Edge,
    direction: &str,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Option<LinkEntry> {
    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get target note title if available
    let title = index.get_metadata(&edge.to).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: direction.to_string(),
        id: edge.to.clone(),
        title,
        link_type: edge.link_type.to_string(),
        source: edge.source.to_string(),
        via: None,
    })
}

/// Filter and convert an inbound edge to a LinkEntry
pub fn filter_and_convert_inbound(
    edge: &Edge,
    index: &Index,
    store: &Store,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
    virtual_inversion: bool,
) -> Option<LinkEntry> {
    // If virtual inversion is requested, we treat this inbound edge as a virtual outbound edge
    if virtual_inversion {
        let virtual_edge = edge.invert(store.config());
        return filter_and_convert(
            &virtual_edge,
            "out",
            index,
            type_filter,
            typed_only,
            inline_only,
        );
    }

    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get source note title if available
    let title = index.get_metadata(&edge.from).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: "in".to_string(),
        id: edge.from.clone(),
        title,
        link_type: edge.link_type.to_string(),
        source: edge.source.to_string(),
        via: None,
    })
}

/// Get filtered neighbors for a node
pub fn get_filtered_neighbors<'a>(
    index: &'a Index,
    id: &str,
    opts: &TreeOptions,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Vec<(String, &'a Edge)> {
    let mut neighbors: Vec<(String, &Edge)> = Vec::new();

    // Collect all source IDs that map to this ID (for gathering edges)
    // This includes the ID itself plus any notes compacted by this ID
    let source_ids = equivalence_map
        .and_then(|map| map.get(id).cloned())
        .unwrap_or_else(|| vec![id.to_string()]);

    // Get outbound edges from ALL source IDs
    if opts.direction == Direction::Out || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_outbound_edges(source_id) {
                if filter_edge(edge, opts) {
                    neighbors.push((edge.to.clone(), edge));
                }
            }
        }
    }

    // Get inbound edges to ALL source IDs (backlinks)
    if opts.direction == Direction::In || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_inbound_edges(source_id) {
                if filter_edge(edge, opts) {
                    neighbors.push((edge.from.clone(), edge));
                }
            }
        }
    }

    // Sort for determinism: edge type, then target id
    neighbors.sort_by(|a, b| {
        a.1.link_type
            .cmp(&b.1.link_type)
            .then_with(|| a.0.cmp(&b.0))
    });

    neighbors
}

/// Check if an edge passes the filters
pub fn filter_edge(edge: &Edge, opts: &TreeOptions) -> bool {
    crate::lib::graph::types::filter_edge(edge, opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_parsing() {
        assert_eq!("out".parse::<Direction>().unwrap(), Direction::Out);
        assert_eq!("in".parse::<Direction>().unwrap(), Direction::In);
        assert_eq!("both".parse::<Direction>().unwrap(), Direction::Both);
        assert_eq!("OUT".parse::<Direction>().unwrap(), Direction::Out);
    }

    #[test]
    fn test_direction_parsing_invalid() {
        assert!("invalid".parse::<Direction>().is_err());
    }

    #[test]
    fn test_tree_options_default() {
        let opts = TreeOptions::default();
        assert_eq!(opts.direction, Direction::Both);
        assert_eq!(opts.max_hops.as_u32_for_display(), 3);
        assert!(opts.type_include.is_empty());
        assert!(opts.type_exclude.is_empty());
        assert!(!opts.typed_only);
        assert!(!opts.inline_only);
        assert!(opts.max_nodes.is_none());
        assert!(opts.max_edges.is_none());
        assert!(opts.max_fanout.is_none());
    }
}
