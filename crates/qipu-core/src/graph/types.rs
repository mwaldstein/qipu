use crate::index::{Edge, LinkSource};
use crate::note::NoteType;
use serde::Serialize;

/// Represents the cost of traversing a single edge
/// For v1, all edges have cost 1.0, but this type supports
/// future per-link-type cost configuration (e.g., part-of = 0.5)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HopCost(f32);

impl HopCost {
    pub const DEFAULT: HopCost = HopCost(1.0);

    pub fn new(cost: f32) -> Self {
        HopCost(cost)
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn as_u32_for_display(&self) -> u32 {
        self.0 as u32
    }
}

impl Default for HopCost {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl std::ops::Add for HopCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        HopCost(self.0 + other.0)
    }
}

impl From<u32> for HopCost {
    fn from(hops: u32) -> Self {
        HopCost(hops as f32)
    }
}

/// Get the hop cost for a given link type
/// Uses config for per-type costs, with defaults for standard types
pub fn get_link_type_cost(link_type: &str, config: &crate::config::StoreConfig) -> HopCost {
    HopCost::new(config.get_link_cost(link_type))
}

/// Get the edge cost for traversing to a target note
/// Formula: `LinkTypeCost * (1 + (100 - value) / 100)`
/// - Value 100 → multiplier 1.0 (no penalty)
/// - Value 50 → multiplier 1.5
/// - Value 0 → multiplier 2.0 (maximum penalty)
///
/// # Arguments
/// * `link_type` - The type of link being traversed
/// * `target_value` - The value of the target note (0-100)
/// * `config` - Store configuration for per-type costs
///
/// # Returns
/// The total cost to traverse this edge
pub fn get_edge_cost(
    link_type: &str,
    target_value: u8,
    config: &crate::config::StoreConfig,
) -> HopCost {
    let link_cost = get_link_type_cost(link_type, config);
    let value_multiplier = 1.0 + (100 - target_value) as f32 / 100.0;
    HopCost(link_cost.value() * value_multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hop_cost_default() {
        let cost = HopCost::DEFAULT;
        assert_eq!(cost.value(), 1.0);
        assert_eq!(cost.as_u32_for_display(), 1);
    }

    #[test]
    fn test_hop_cost_from_u32() {
        let cost = HopCost::from(5);
        assert_eq!(cost.value(), 5.0);
        assert_eq!(cost.as_u32_for_display(), 5);
    }

    #[test]
    fn test_hop_cost_addition() {
        let cost1 = HopCost::from(2);
        let cost2 = HopCost::from(3);
        let sum = cost1 + cost2;
        assert_eq!(sum.value(), 5.0);
    }

    #[test]
    fn test_hop_cost_fractional() {
        let cost1 = HopCost::new(1.5);
        let cost2 = HopCost::new(2.5);
        let sum = cost1 + cost2;
        assert_eq!(sum.value(), 4.0);
        assert_eq!(sum.as_u32_for_display(), 4);
    }

    #[test]
    fn test_get_link_type_cost_default() {
        let config = crate::config::StoreConfig::default();
        let cost = get_link_type_cost("supports", &config);
        assert_eq!(cost.value(), 1.0);
    }

    #[test]
    fn test_get_link_type_cost_unknown() {
        let config = crate::config::StoreConfig::default();
        let cost = get_link_type_cost("unknown-type", &config);
        assert_eq!(cost.value(), 1.0);
    }

    #[test]
    fn test_get_link_type_cost_standard_structural() {
        let config = crate::config::StoreConfig::default();
        let cost = get_link_type_cost("part-of", &config);
        assert_eq!(cost.value(), 0.5);
    }

    #[test]
    fn test_get_link_type_cost_identity() {
        let config = crate::config::StoreConfig::default();
        let cost = get_link_type_cost("same-as", &config);
        assert_eq!(cost.value(), 0.5);
    }

    #[test]
    fn test_get_edge_cost_max_value() {
        let config = crate::config::StoreConfig::default();
        let cost = get_edge_cost("supports", 100, &config);
        assert_eq!(cost.value(), 1.0);
    }

    #[test]
    fn test_get_edge_cost_mid_value() {
        let config = crate::config::StoreConfig::default();
        let cost = get_edge_cost("supports", 50, &config);
        assert_eq!(cost.value(), 1.5);
    }

    #[test]
    fn test_get_edge_cost_min_value() {
        let config = crate::config::StoreConfig::default();
        let cost = get_edge_cost("supports", 0, &config);
        assert_eq!(cost.value(), 2.0);
    }

    #[test]
    fn test_get_edge_cost_custom_link_type() {
        let config = crate::config::StoreConfig::default();
        let cost = get_edge_cost("part-of", 75, &config);
        assert_eq!(cost.value(), 0.625); // 0.5 * 1.25
    }

    #[test]
    fn test_get_edge_cost_boundary() {
        let config = crate::config::StoreConfig::default();
        let cost = get_edge_cost("supports", 1, &config);
        let expected = 1.0 + (100.0 - 1.0) / 100.0;
        assert!((cost.value() - expected).abs() < 0.001);
    }

    #[test]
    fn test_tree_options_default_max_hops() {
        let opts = TreeOptions::default();
        assert_eq!(opts.max_hops.value(), 3.0);
        assert_eq!(opts.max_hops.as_u32_for_display(), 3);
    }

    #[test]
    fn test_tree_options_custom_max_hops() {
        let opts = TreeOptions {
            max_hops: HopCost::from(5),
            ..Default::default()
        };
        assert_eq!(opts.max_hops.value(), 5.0);
    }

    #[test]
    fn test_tree_options_ignore_value_default() {
        let opts = TreeOptions::default();
        assert!(!opts.ignore_value);
    }

    #[test]
    fn test_tree_options_ignore_value_true() {
        let opts = TreeOptions {
            ignore_value: true,
            ..Default::default()
        };
        assert!(opts.ignore_value);
    }

    #[test]
    fn test_custom_link_cost() {
        let mut config = crate::config::StoreConfig::default();
        config.set_link_cost("custom-type", 0.8);
        let cost = get_link_type_cost("custom-type", &config);
        assert_eq!(cost.value(), 0.8);
    }

    #[test]
    fn test_custom_link_cost_overrides_default() {
        let mut config = crate::config::StoreConfig::default();
        config.set_link_cost("part-of", 0.9); // Override default 0.5
        let cost = get_link_type_cost("part-of", &config);
        assert_eq!(cost.value(), 0.9);
    }

    #[test]
    fn test_get_edge_cost_with_custom_type() {
        let mut config = crate::config::StoreConfig::default();
        config.set_link_cost("custom", 0.8);
        let cost = get_edge_cost("custom", 50, &config);
        assert_eq!(cost.value(), 1.2); // 0.8 * 1.5
    }
}

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
    /// Maximum traversal depth (as hop cost)
    pub max_hops: HopCost,
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
    /// Filter by minimum value (0-100, None = no filter)
    pub min_value: Option<u8>,
    /// Ignore note values when calculating edge costs (use unweighted BFS)
    pub ignore_value: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        TreeOptions {
            direction: Direction::Both,
            max_hops: HopCost::from(3),
            type_include: Vec::new(),
            type_exclude: Vec::new(),
            typed_only: false,
            inline_only: false,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
            semantic_inversion: true,
            min_value: None,
            ignore_value: false,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

/// Link in the traversal output
#[derive(Debug, Clone, Serialize)]
pub struct TreeLink {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
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
