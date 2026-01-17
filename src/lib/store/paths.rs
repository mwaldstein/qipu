use crate::lib::error::{QipuError, Result};
use std::path::{Path, PathBuf};

/// Default store directory name (hidden)
pub const DEFAULT_STORE_DIR: &str = ".qipu";

/// Visible store directory name
pub const VISIBLE_STORE_DIR: &str = "qipu";

/// Store subdirectories
pub const NOTES_DIR: &str = "notes";
pub const MOCS_DIR: &str = "mocs";
pub const ATTACHMENTS_DIR: &str = "attachments";
pub const TEMPLATES_DIR: &str = "templates";
pub const CACHE_DIR: &str = ".cache";

/// Configuration filename
pub const CONFIG_FILE: &str = "config.toml";

/// Workspace metadata filename
pub const WORKSPACE_FILE: &str = "workspace.toml";

/// Gitignore filename
pub const GITIGNORE_FILE: &str = ".gitignore";

/// Workspaces subdirectory
pub const WORKSPACES_DIR: &str = "workspaces";

pub fn discover_store(root: &Path) -> Result<PathBuf> {
    let mut current = root.to_path_buf();

    loop {
        // Check for default hidden store
        let store_path = current.join(DEFAULT_STORE_DIR);
        if store_path.is_dir() {
            return Ok(store_path);
        }

        // Check for visible store
        let visible_path = current.join(VISIBLE_STORE_DIR);
        if visible_path.is_dir() {
            return Ok(visible_path);
        }

        // Move up to parent directory
        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
            }
            _ => {
                // Reached filesystem root
                return Err(QipuError::StoreNotFound {
                    search_root: root.to_path_buf(),
                });
            }
        }
    }
}
