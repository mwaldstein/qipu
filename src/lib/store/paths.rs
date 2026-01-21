use crate::lib::config::StoreConfig;
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

/// Configuration filename
pub const CONFIG_FILE: &str = "config.toml";

/// Workspace metadata filename
pub const WORKSPACE_FILE: &str = "workspace.toml";

/// Gitignore filename
pub const GITIGNORE_FILE: &str = ".gitignore";

/// Workspaces subdirectory
pub const WORKSPACES_DIR: &str = "workspaces";

/// Project root markers that stop upward discovery
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "Cargo.toml",
    "package.json",
    "go.mod",
    "pyproject.toml",
];

fn is_project_root(dir: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| dir.join(marker).exists())
}

/// Try to load config and check for custom store_path
fn check_config_for_custom_path(store_dir: &Path) -> Option<PathBuf> {
    let config_path = store_dir.join(CONFIG_FILE);
    if config_path.exists() {
        if let Ok(config) = StoreConfig::load(&config_path) {
            if let Some(ref store_path) = config.store_path {
                return Some(PathBuf::from(store_path));
            }
        }
    }
    None
}

pub fn discover_store(root: &Path) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    let mut passed_project_root = false;

    loop {
        // Check for default hidden store
        let store_path = current.join(DEFAULT_STORE_DIR);
        if store_path.is_dir() {
            if let Some(custom_path) = check_config_for_custom_path(&store_path) {
                let resolved_path = if custom_path.is_absolute() {
                    custom_path.clone()
                } else {
                    // Relative path is relative to the project root (current directory)
                    current.join(&custom_path)
                };
                if resolved_path.is_dir() {
                    return Ok(resolved_path);
                }
            }
            return Ok(store_path);
        }

        // Check for visible store
        let visible_path = current.join(VISIBLE_STORE_DIR);
        if visible_path.is_dir() {
            if let Some(custom_path) = check_config_for_custom_path(&visible_path) {
                let resolved_path = if custom_path.is_absolute() {
                    custom_path.clone()
                } else {
                    // Relative path is relative to the project root (current directory)
                    current.join(&custom_path)
                };
                if resolved_path.is_dir() {
                    return Ok(resolved_path);
                }
            }
            return Ok(visible_path);
        }

        // Check if this directory is a project root
        if is_project_root(&current) {
            passed_project_root = true;
        }

        // Move up to parent directory
        match current.parent() {
            Some(parent) if parent != current => {
                // Stop if we already passed a project root
                if passed_project_root {
                    return Err(QipuError::StoreNotFound {
                        search_root: root.to_path_buf(),
                    });
                }
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
