use crate::lib::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub temporary: bool,
    pub parent_id: Option<String>,
}

impl WorkspaceMetadata {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let metadata: WorkspaceMetadata = toml::from_str(&content).map_err(|e| {
            crate::lib::error::QipuError::Other(format!("failed to parse workspace.toml: {}", e))
        })?;
        Ok(metadata)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::lib::error::QipuError::Other(format!(
                "failed to serialize workspace.toml: {}",
                e
            ))
        })?;
        fs::write(path, content)?;
        Ok(())
    }
}
