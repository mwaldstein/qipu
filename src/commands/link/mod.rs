//! Link management commands for qipu
//!
//! Per spec (specs/cli-interface.md, specs/graph-traversal.md):
//! - `qipu link list <id>` - list links for a note
//! - `qipu link add <from> <to> --type <t>` - add typed link
//! - `qipu link remove <from> <to> --type <t>` - remove typed link
//! - `qipu link tree <id>` - traversal tree from note
//! - `qipu link path <from> <to>` - find path between notes

pub mod add;
pub mod list;
pub mod path;
pub mod remove;
pub mod tree;

use std::collections::HashMap;

use crate::lib::error::Result;
use crate::lib::index::{Edge, Index, LinkSource};
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use serde::Serialize;

/// Direction for link listing/traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Outbound links only (links FROM this note)
    Out,
    /// Inbound links only (backlinks TO this note)
    In,
    #[default]
    /// Both directions
    Both,
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "out" => Ok(Direction::Out),
            "in" => Ok(Direction::In),
            "both" => Ok(Direction::Both),
            other => Err(format!(
                "unknown direction '{}' (expected: out, in, both)",
                other
            )),
        }
    }
}

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
}

/// Options for tree traversal
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Direction for traversal
    pub direction: Direction,
    /// Maximum traversal depth
    pub max_hops: u32,
    /// Include only these link types (empty = all)
    pub type_include: Vec<String>,
    /// Exclude these link types
    pub type_exclude: Vec<String>,
    /// Show only typed links
    pub typed_only: bool,
    /// Show only inline links
    pub inline_only: bool,
    /// Maximum nodes to visit
    pub max_nodes: Option<usize>,
    /// Maximum edges to emit
    pub max_edges: Option<usize>,
    /// Maximum neighbors per node
    pub max_fanout: Option<usize>,
    /// Maximum output characters (records format only)
    pub max_chars: Option<usize>,
}

impl Default for TreeOptions {
    fn default() -> Self {
        TreeOptions {
            direction: Direction::Both,
            max_hops: 3,
            type_include: Vec::new(),
            type_exclude: Vec::new(),
            typed_only: false,
            inline_only: false,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
        }
    }
}

/// Note in the traversal output
#[derive(Debug, Clone, Serialize)]
pub struct TreeNote {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub tags: Vec<String>,
    pub path: String,
}

/// Link in the traversal output
#[derive(Debug, Clone, Serialize)]
pub struct TreeLink {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub source: String,
}

/// Spanning tree entry
#[derive(Debug, Clone, Serialize)]
pub struct SpanningTreeEntry {
    pub from: String,
    pub to: String,
    pub hop: u32,
}

/// Complete traversal result
#[derive(Debug, Clone, Serialize)]
pub struct TreeResult {
    pub root: String,
    pub direction: String,
    pub max_hops: u32,
    pub truncated: bool,
    pub truncation_reason: Option<String>,
    pub notes: Vec<TreeNote>,
    pub links: Vec<TreeLink>,
    pub spanning_tree: Vec<SpanningTreeEntry>,
}

/// Path result
#[derive(Debug, Clone, Serialize)]
pub struct PathResult {
    pub from: String,
    pub to: String,
    pub direction: String,
    pub found: bool,
    pub notes: Vec<TreeNote>,
    pub links: Vec<TreeLink>,
    pub path_length: usize,
}

/// Resolve a note ID or path to a canonical note ID
pub fn resolve_note_id(store: &Store, id_or_path: &str) -> Result<String> {
    // If it looks like an ID (starts with qp-), try to use it directly
    if id_or_path.starts_with("qp-") {
        // Could be a full filename like qp-xxxx-slug.md or just qp-xxxx
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
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
    })
}

/// Filter and convert an inbound edge to a LinkEntry
pub fn filter_and_convert_inbound(
    edge: &Edge,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
    virtual_inversion: bool,
) -> Option<LinkEntry> {
    // If virtual inversion is requested, we treat this inbound edge as a virtual outbound edge
    if virtual_inversion {
        let virtual_edge = edge.invert();
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
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
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
    // Source filter
    if opts.typed_only && edge.source != LinkSource::Typed {
        return false;
    }
    if opts.inline_only && edge.source != LinkSource::Inline {
        return false;
    }

    // Type inclusion filter
    if !opts.type_include.is_empty() && !opts.type_include.contains(&edge.link_type) {
        return false;
    }

    // Type exclusion filter
    if opts.type_exclude.contains(&edge.link_type) {
        return false;
    }

    true
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
        assert_eq!(opts.max_hops, 3);
        assert!(opts.type_include.is_empty());
        assert!(opts.type_exclude.is_empty());
        assert!(!opts.typed_only);
        assert!(!opts.inline_only);
        assert!(opts.max_nodes.is_none());
        assert!(opts.max_edges.is_none());
        assert!(opts.max_fanout.is_none());
    }
}
