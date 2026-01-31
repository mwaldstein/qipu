//! Configuration type definitions

use crate::id::IdScheme;
use crate::note::NoteType;
use serde::{Deserialize, Serialize};

/// Current store format version
pub const STORE_FORMAT_VERSION: u32 = 1;

/// Store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    /// Store format version for compatibility checking
    #[serde(default = "default_version")]
    pub version: u32,

    /// Default note type for new notes
    #[serde(default)]
    pub default_note_type: NoteType,

    /// ID generation scheme
    #[serde(default)]
    pub id_scheme: IdScheme,

    /// Editor override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,

    /// Git branch for protected branch workflow (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Custom store root path (optional, overrides default discovery)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_path: Option<String>,

    /// Rewrite wiki-links to markdown links during index (optional; opt-in)
    #[serde(default)]
    pub rewrite_wiki_links: bool,

    /// Enable stemming for similarity matching (optional; default true)
    #[serde(default = "default_stemming")]
    pub stemming: bool,

    /// Tag aliases: short aliases mapped to canonical tag names
    #[serde(default)]
    pub tag_aliases: std::collections::HashMap<String, String>,

    /// Graph configuration
    #[serde(default)]
    pub graph: GraphConfig,

    /// Auto-indexing configuration
    #[serde(default)]
    pub auto_index: AutoIndexConfig,

    /// Search ranking configuration
    #[serde(default)]
    pub search: SearchConfig,

    /// Custom ontology configuration
    #[serde(default)]
    pub ontology: OntologyConfig,
}

/// Configuration for graph traversal and link types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Custom link type definitions
    #[serde(default)]
    pub types: std::collections::HashMap<String, LinkTypeConfig>,
}

/// Configuration for auto-indexing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoIndexConfig {
    /// Enable/disable auto-indexing
    #[serde(default = "default_auto_index_enabled")]
    pub enabled: bool,

    /// Indexing strategy: "adaptive", "full", "incremental", "quick"
    #[serde(default = "default_auto_index_strategy")]
    pub strategy: String,

    /// Note count threshold for adaptive strategy
    #[serde(default = "default_adaptive_threshold")]
    pub adaptive_threshold: usize,

    /// Notes to include in quick mode
    #[serde(default = "default_quick_notes")]
    pub quick_notes: usize,
}

/// Configuration for search ranking parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Recency boost numerator (default 0.1)
    #[serde(default = "default_recency_boost_numerator")]
    pub recency_boost_numerator: f64,

    /// Recency decay in days (default 7.0)
    #[serde(default = "default_recency_decay_days")]
    pub recency_decay_days: f64,
}

/// Configuration for a single note type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteTypeConfig {
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Usage guidance for LLMs
    #[serde(default)]
    pub usage: Option<String>,
}

/// Configuration for a single link type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkTypeConfig {
    /// Inverse link type
    #[serde(default)]
    pub inverse: Option<String>,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Hop cost for traversing this link type (default 1.0)
    #[serde(default = "default_link_cost")]
    pub cost: f32,

    /// Usage guidance for LLMs
    #[serde(default)]
    pub usage: Option<String>,
}

/// Ontology resolution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OntologyMode {
    /// Use only standard ontology (built-in note and link types)
    #[default]
    Default,
    /// Extend standard ontology with custom types
    Extended,
    /// Replace standard ontology with custom types only
    Replacement,
}

impl std::fmt::Display for OntologyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OntologyMode::Default => write!(f, "default"),
            OntologyMode::Extended => write!(f, "extended"),
            OntologyMode::Replacement => write!(f, "replacement"),
        }
    }
}

/// Custom ontology configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OntologyConfig {
    /// Resolution mode for custom types
    #[serde(default)]
    pub mode: OntologyMode,

    /// Custom note type definitions
    #[serde(default)]
    pub note_types: std::collections::HashMap<String, NoteTypeConfig>,

    /// Custom link type definitions
    #[serde(default)]
    pub link_types: std::collections::HashMap<String, LinkTypeConfig>,
}

fn default_version() -> u32 {
    STORE_FORMAT_VERSION
}

fn default_stemming() -> bool {
    true
}

fn default_link_cost() -> f32 {
    1.0
}

fn default_auto_index_enabled() -> bool {
    true
}

fn default_auto_index_strategy() -> String {
    "adaptive".to_string()
}

fn default_adaptive_threshold() -> usize {
    10000
}

fn default_quick_notes() -> usize {
    100
}

fn default_recency_boost_numerator() -> f64 {
    0.1
}

fn default_recency_decay_days() -> f64 {
    7.0
}

impl Default for AutoIndexConfig {
    fn default() -> Self {
        AutoIndexConfig {
            enabled: default_auto_index_enabled(),
            strategy: default_auto_index_strategy(),
            adaptive_threshold: default_adaptive_threshold(),
            quick_notes: default_quick_notes(),
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            recency_boost_numerator: default_recency_boost_numerator(),
            recency_decay_days: default_recency_decay_days(),
        }
    }
}

/// Get the standard cost for a known link type
/// Returns None for unknown/custom types (use default 1.0)
pub fn get_standard_link_cost(link_type: &str) -> Option<f32> {
    match link_type {
        // Structural types (reduced cost for strong cohesion)
        "part-of" | "has-part" | "follows" | "precedes" => Some(0.5),

        // Identity types (reduced cost for unification)
        "same-as" | "alias-of" | "has-alias" => Some(0.5),

        // Argumentative types (standard cost)
        "supports" | "supported-by" | "contradicts" | "contradicted-by" | "answers"
        | "answered-by" | "refines" | "refined-by" | "related" => Some(1.0),

        // Unknown types - use default
        _ => None,
    }
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            version: STORE_FORMAT_VERSION,
            default_note_type: NoteType::default(),
            id_scheme: IdScheme::default(),
            editor: None,
            branch: None,
            store_path: None,
            rewrite_wiki_links: false,
            stemming: default_stemming(),
            tag_aliases: std::collections::HashMap::new(),
            graph: GraphConfig::default(),
            auto_index: AutoIndexConfig::default(),
            search: SearchConfig::default(),
            ontology: OntologyConfig::default(),
        }
    }
}
