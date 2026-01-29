//! Store configuration for qipu
//!
//! Configuration is stored in `.qipu/config.toml` per spec (specs/storage-format.md).

pub mod types;

use std::fs;
use std::path::Path;

use crate::error::{QipuError, Result};
use crate::ontology::Ontology;

#[allow(unused_imports)]
pub use types::{
    AutoIndexConfig, GraphConfig, LinkTypeConfig, NoteTypeConfig, OntologyConfig, OntologyMode,
    SearchConfig, StoreConfig, STORE_FORMAT_VERSION,
};

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
        if let Some(cost) = types::get_standard_link_cost(link_type) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::STORE_FORMAT_VERSION;
    use crate::id::IdScheme;
    use crate::note::NoteType;
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
