use crate::lib::index::{Edge, LinkSource};
use crate::lib::note::NoteType;
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
    /// Whether to use semantic inversion for inbound links
    pub semantic_inversion: bool,
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
            semantic_inversion: true,
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
    #[serde(rename = "type")]
    pub link_type: String,
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

/// Helper function to filter an edge based on TreeOptions
pub fn filter_edge(edge: &Edge, opts: &TreeOptions) -> bool {
    // Source filter
    if opts.typed_only && edge.source != LinkSource::Typed {
        return false;
    }
    if opts.inline_only && edge.source != LinkSource::Inline {
        return false;
    }

    // Type inclusion filter
    if !opts.type_include.is_empty()
        && !opts
            .type_include
            .iter()
            .any(|t| t == edge.link_type.as_str())
    {
        return false;
    }

    // Type exclusion filter
    if opts
        .type_exclude
        .iter()
        .any(|t| t == edge.link_type.as_str())
    {
        return false;
    }

    true
}
