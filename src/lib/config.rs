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

    /// Custom link type definitions
    #[serde(default)]
    pub links: LinkConfig,
}

/// Configuration for link types and behaviors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkConfig {
    /// Custom link inversions (forward -> inverse)
    #[serde(default)]
    pub inverses: std::collections::HashMap<String, String>,
}

impl StoreConfig {
    /// Get the inverse of a link type, falling back to standard ontology or default pattern
    pub fn get_inverse(&self, link_type: &str) -> String {
        // 1. Check user-defined inverses
        if let Some(inv) = self.links.inverses.get(link_type) {
            return inv.clone();
        }

        // 2. Check standard ontology
        crate::lib::note::LinkType::new(link_type)
            .inverse()
            .to_string()
    }
}

fn default_version() -> u32 {
    STORE_FORMAT_VERSION
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            version: STORE_FORMAT_VERSION,
            default_note_type: NoteType::default(),
            id_scheme: IdScheme::default(),
            editor: None,
            branch: None,
        }
    }
}

impl StoreConfig {
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
    }
}
