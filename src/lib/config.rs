//! Store configuration for qipu
//!
//! Configuration is stored in `.qipu/config.toml` per spec (specs/storage-format.md).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::lib::error::{QipuError, Result};
use crate::lib::id::IdScheme;
use crate::lib::note::NoteType;

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

    /// Graph configuration
    #[serde(default)]
    pub graph: GraphConfig,
}

/// Configuration for graph traversal and link types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Custom link type definitions
    #[serde(default)]
    pub types: std::collections::HashMap<String, LinkTypeConfig>,
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
}

impl StoreConfig {
    /// Get the inverse of a link type, falling back to standard ontology or default pattern
    pub fn get_inverse(&self, link_type: &str) -> String {
        // 1. Check user-defined inverses
        if let Some(type_config) = self.graph.types.get(link_type) {
            if let Some(ref inv) = type_config.inverse {
                return inv.clone();
            }
        }

        // 2. Check standard ontology
        crate::lib::note::LinkType::new(link_type)
            .inverse()
            .to_string()
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

    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: StoreConfig = toml::from_str(&content)?;
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
            graph: GraphConfig::default(),
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
        assert_eq!(config.default_note_type, NoteType::Fleeting);
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
}
