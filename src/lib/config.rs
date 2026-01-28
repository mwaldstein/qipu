//! Store configuration for qipu
//!
//! Configuration is stored in `.qipu/config.toml` per spec (specs/storage-format.md).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::lib::error::{QipuError, Result};
use crate::lib::id::IdScheme;
use crate::lib::note::NoteType;
use crate::lib::ontology::Ontology;

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

impl StoreConfig {
    /// Get the inverse of a link type using the configured ontology
    pub fn get_inverse(&self, link_type: &str) -> String {
        let ontology = Ontology::from_config_with_graph(&self.ontology, &self.graph);
        ontology.get_inverse(link_type)
    }

    /// Validate a note type using the configured ontology
    pub fn validate_note_type(&self, note_type: &str) -> Result<()> {
        let ontology = Ontology::from_config_with_graph(&self.ontology, &self.graph);
        ontology.validate_note_type(note_type)
    }

    /// Validate a link type using the configured ontology
    pub fn validate_link_type(&self, link_type: &str) -> Result<()> {
        let ontology = Ontology::from_config_with_graph(&self.ontology, &self.graph);
        ontology.validate_link_type(link_type)
    }

    /// Get the hop cost for a link type
    /// Returns user-defined cost, or standard type cost, or default (1.0)
    pub fn get_link_cost(&self, link_type: &str) -> f32 {
        // 1. Check user-defined costs
        if let Some(type_config) = self.graph.types.get(link_type) {
            return type_config.cost;
        }

        // 2. Check standard type costs
        if let Some(cost) = get_standard_link_cost(link_type) {
            return cost;
        }

        // 3. Default cost
        1.0
    }

    /// Set a custom cost for a link type
    #[allow(dead_code)]
    pub fn set_link_cost(&mut self, link_type: &str, cost: f32) {
        self.graph
            .types
            .entry(link_type.to_string())
            .or_default()
            .cost = cost;
    }

    /// Resolve a tag alias to its canonical name
    /// Returns: canonical tag name, or the original tag if no alias exists
    #[allow(dead_code)]
    pub fn resolve_tag_alias(&self, tag: &str) -> String {
        self.tag_aliases
            .get(tag)
            .cloned()
            .unwrap_or_else(|| tag.to_string())
    }

    /// Get all equivalent tags for a given tag (including the tag itself and any aliases)
    /// This is useful for filtering: if user queries with "ml", match notes tagged "ml" OR "machine-learning"
    pub fn get_equivalent_tags(&self, tag: &str) -> Vec<String> {
        let mut tags = vec![tag.to_string()];

        // If this tag is an alias, add the canonical tag
        if let Some(canonical) = self.tag_aliases.get(tag) {
            tags.push(canonical.clone());
        }

        // If this is a canonical tag, add all its aliases
        for (alias, canonical) in &self.tag_aliases {
            if canonical == tag {
                tags.push(alias.clone());
            }
        }

        tags.sort();
        tags.dedup();
        tags
    }

    /// Add a tag alias
    #[allow(dead_code)]
    pub fn add_tag_alias(&mut self, alias: String, canonical: String) {
        self.tag_aliases.insert(alias, canonical);
    }

    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: StoreConfig = toml::from_str(&content)?;

        if !config.graph.types.is_empty() {
            for name in config.graph.types.keys() {
                eprintln!(
                    "Warning: Deprecated configuration [graph.types.{}] - use [ontology.link_types.{}] instead",
                    name, name
                );
            }
        }

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| QipuError::Other(format!("failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Create default configuration with sensible defaults
    #[allow(dead_code)]
    pub fn with_defaults() -> Self {
        Self::default()
    }
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
fn get_standard_link_cost(link_type: &str) -> Option<f32> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = StoreConfig::default();
        assert_eq!(config.version, STORE_FORMAT_VERSION);
        assert_eq!(config.default_note_type, NoteType::from(NoteType::FLEETING));
        assert_eq!(config.id_scheme, IdScheme::Hash);
        assert!(config.editor.is_none());
        assert!(config.store_path.is_none());
        assert!(!config.rewrite_wiki_links);
        assert!(config.stemming); // Default to true
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = StoreConfig::default();
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(loaded.version, config.version);
        assert_eq!(loaded.default_note_type, config.default_note_type);
        assert!(loaded.store_path.is_none());
    }

    #[test]
    fn test_save_and_load_with_store_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = StoreConfig {
            store_path: Some("data/notes".to_string()),
            ..Default::default()
        };
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(loaded.store_path, Some("data/notes".to_string()));
    }

    #[test]
    fn test_save_and_load_with_absolute_store_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = StoreConfig {
            store_path: Some("/absolute/path/to/store".to_string()),
            ..Default::default()
        };
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(
            loaded.store_path,
            Some("/absolute/path/to/store".to_string())
        );
    }

    #[test]
    fn test_stemming_config_defaults_to_true() {
        let config = StoreConfig::default();
        assert!(config.stemming);
    }

    #[test]
    fn test_stemming_config_can_be_disabled() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = StoreConfig {
            stemming: false,
            ..Default::default()
        };
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert!(!loaded.stemming);
    }

    #[test]
    fn test_get_link_cost_default() {
        let config = StoreConfig::default();
        assert_eq!(config.get_link_cost("supports"), 1.0);
        assert_eq!(config.get_link_cost("unknown"), 1.0);
    }

    #[test]
    fn test_get_link_cost_standard_structural() {
        let config = StoreConfig::default();
        assert_eq!(config.get_link_cost("part-of"), 0.5);
        assert_eq!(config.get_link_cost("has-part"), 0.5);
        assert_eq!(config.get_link_cost("follows"), 0.5);
        assert_eq!(config.get_link_cost("precedes"), 0.5);
    }

    #[test]
    fn test_get_link_cost_identity() {
        let config = StoreConfig::default();
        assert_eq!(config.get_link_cost("same-as"), 0.5);
        assert_eq!(config.get_link_cost("alias-of"), 0.5);
        assert_eq!(config.get_link_cost("has-alias"), 0.5);
    }

    #[test]
    fn test_get_link_cost_argumentative() {
        let config = StoreConfig::default();
        assert_eq!(config.get_link_cost("supports"), 1.0);
        assert_eq!(config.get_link_cost("supported-by"), 1.0);
        assert_eq!(config.get_link_cost("contradicts"), 1.0);
        assert_eq!(config.get_link_cost("answers"), 1.0);
        assert_eq!(config.get_link_cost("refines"), 1.0);
        assert_eq!(config.get_link_cost("related"), 1.0);
    }

    #[test]
    fn test_set_link_cost() {
        let mut config = StoreConfig::default();
        config.set_link_cost("custom-type", 0.8);
        assert_eq!(config.get_link_cost("custom-type"), 0.8);
    }

    #[test]
    fn test_set_link_cost_overrides_standard() {
        let mut config = StoreConfig::default();
        config.set_link_cost("part-of", 0.9);
        assert_eq!(config.get_link_cost("part-of"), 0.9);
    }

    #[test]
    fn test_link_cost_serialization() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = StoreConfig::default();
        config.set_link_cost("custom", 0.7);
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(loaded.get_link_cost("custom"), 0.7);
    }

    #[test]
    fn test_tag_aliases_default_empty() {
        let config = StoreConfig::default();
        assert!(config.tag_aliases.is_empty());
    }

    #[test]
    fn test_add_tag_alias() {
        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());
        config.add_tag_alias("ai".to_string(), "artificial-intelligence".to_string());

        assert_eq!(config.tag_aliases.len(), 2);
        assert_eq!(
            config.tag_aliases.get("ml"),
            Some(&"machine-learning".to_string())
        );
        assert_eq!(
            config.tag_aliases.get("ai"),
            Some(&"artificial-intelligence".to_string())
        );
    }

    #[test]
    fn test_resolve_tag_alias() {
        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());

        assert_eq!(config.resolve_tag_alias("ml"), "machine-learning");
        assert_eq!(config.resolve_tag_alias("ai"), "ai"); // no alias, returns original
    }

    #[test]
    fn test_tag_aliases_serialization() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(loaded.resolve_tag_alias("ml"), "machine-learning");
    }

    #[test]
    fn test_get_equivalent_tags_alias() {
        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());

        let equiv = config.get_equivalent_tags("ml");
        assert_eq!(equiv, vec!["machine-learning", "ml"]);
    }

    #[test]
    fn test_get_equivalent_tags_canonical() {
        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());

        let equiv = config.get_equivalent_tags("machine-learning");
        assert_eq!(equiv, vec!["machine-learning", "ml"]);
    }

    #[test]
    fn test_get_equivalent_tags_no_alias() {
        let config = StoreConfig::default();

        let equiv = config.get_equivalent_tags("some-tag");
        assert_eq!(equiv, vec!["some-tag"]);
    }

    #[test]
    fn test_get_equivalent_tags_multiple_aliases() {
        let mut config = StoreConfig::default();
        config.add_tag_alias("ml".to_string(), "machine-learning".to_string());
        config.add_tag_alias("ai".to_string(), "machine-learning".to_string());

        let equiv = config.get_equivalent_tags("machine-learning");
        assert_eq!(equiv, vec!["ai", "machine-learning", "ml"]);
    }

    #[test]
    fn test_search_config_defaults() {
        let config = StoreConfig::default();
        assert_eq!(config.search.recency_boost_numerator, 0.1);
        assert_eq!(config.search.recency_decay_days, 7.0);
    }

    #[test]
    fn test_search_config_serialization() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = StoreConfig::default();
        config.save(&path).unwrap();

        let loaded = StoreConfig::load(&path).unwrap();
        assert_eq!(loaded.search.recency_boost_numerator, 0.1);
        assert_eq!(loaded.search.recency_decay_days, 7.0);
    }
}
