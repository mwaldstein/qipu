//! Global configuration for qipu (stored in ~/.config/qipu/config.toml)

use std::fs;
use std::path::PathBuf;

use crate::error::{QipuError, Result};

const CONFIG_DIR: &str = "qipu";
const CONFIG_FILE: &str = "config.toml";
const CONFIG_DIR_ENV_VAR: &str = "QIPU_CONFIG_DIR";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub telemetry_enabled: bool,
}

impl GlobalConfig {
    fn config_path() -> Result<PathBuf> {
        // Allow environment variable override for testing
        let config_dir = if let Ok(env_dir) = std::env::var(CONFIG_DIR_ENV_VAR) {
            PathBuf::from(env_dir)
        } else {
            dirs::config_dir()
                .ok_or_else(|| {
                    QipuError::Other("unable to determine config directory".to_string())
                })?
                .join(CONFIG_DIR)
        };

        Ok(config_dir.join(CONFIG_FILE))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            QipuError::Other(format!(
                "failed to read global config from {}: {}",
                path.display(),
                e
            ))
        })?;

        toml::from_str(&content).map_err(|e| {
            QipuError::Other(format!(
                "failed to parse global config from {}: {}",
                path.display(),
                e
            ))
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let config_dir = path
            .parent()
            .ok_or_else(|| QipuError::Other("invalid config path".to_string()))?;

        fs::create_dir_all(config_dir).map_err(|e| {
            QipuError::Other(format!(
                "failed to create config directory {}: {}",
                config_dir.display(),
                e
            ))
        })?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| QipuError::Other(format!("failed to serialize config: {}", e)))?;

        fs::write(&path, content).map_err(|e| {
            QipuError::Other(format!(
                "failed to write config to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    pub fn set_telemetry_enabled(&mut self, enabled: bool) {
        self.telemetry_enabled = enabled;
    }

    pub fn get_telemetry_enabled(&self) -> Option<bool> {
        Some(self.telemetry_enabled)
    }

    /// Returns true if the config directory is overridden via environment variable
    pub fn is_config_dir_overridden() -> bool {
        std::env::var(CONFIG_DIR_ENV_VAR).is_ok()
    }

    /// Returns the source description for display purposes
    pub fn source_display() -> String {
        if std::env::var(CONFIG_DIR_ENV_VAR).is_ok() {
            "custom config directory".to_string()
        } else {
            "~/.config/qipu/config.toml".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert!(!config.telemetry_enabled);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut config = GlobalConfig::default();
        config.telemetry_enabled = true;

        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        let loaded =
            toml::from_str::<GlobalConfig>(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(loaded.telemetry_enabled);
    }
}
