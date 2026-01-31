use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub temporary: bool,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceMetadataFile {
    workspace: WorkspaceMetadata,
}

impl WorkspaceMetadata {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let file: WorkspaceMetadataFile = toml::from_str(&content)?;
        Ok(file.workspace)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = WorkspaceMetadataFile {
            workspace: self.clone(),
        };
        let content = toml::to_string_pretty(&file).map_err(|e| {
            crate::error::QipuError::FailedOperation {
                operation: "serialize workspace.toml".to_string(),
                reason: e.to_string(),
            }
        })?;
        fs::write(path, content)?;
        Ok(())
    }
}
