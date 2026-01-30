//! Store management for qipu
//!
//! The store is the root directory containing all qipu data.
//! Default location: `.qipu/` (hidden, git-trackable)

pub mod config;
pub mod io;
mod lifecycle;
pub mod notes;
pub mod paths;
mod query;
pub mod typed_paths;
pub mod workspace;

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::StoreConfig;
use crate::db::Database;
use crate::error::{QipuError, Result};
pub use config::InitOptions;
use paths::{
    ATTACHMENTS_DIR, CONFIG_FILE, DEFAULT_STORE_DIR, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR,
    VISIBLE_STORE_DIR, WORKSPACES_DIR,
};

/// The qipu store
#[derive(Debug)]
pub struct Store {
    /// Root path of the store
    root: PathBuf,
    /// Store configuration
    config: StoreConfig,
    /// SQLite database
    db: Database,
}

impl Store {
    /// Discover a store by walking up from the given root directory
    pub fn discover(root: &Path) -> Result<Self> {
        let store_path = paths::discover_store(root)?;
        Self::open(&store_path)
    }

    /// Open an existing store at the given path
    #[tracing::instrument(skip(path), fields(path = %path.display()))]
    pub fn open(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            return Err(QipuError::StoreNotFound {
                search_root: path.to_path_buf(),
            });
        }

        io::validate_store_layout(path)?;

        let config_path = path.join(CONFIG_FILE);
        let config = if config_path.exists() {
            StoreConfig::load(&config_path)?
        } else {
            // Use default config if missing (per spec: config should have sensible defaults)
            StoreConfig::default()
        };

        // Ensure default templates exist (idempotent, only creates missing ones)
        let templates_dir = path.join(TEMPLATES_DIR);
        io::ensure_default_templates(&templates_dir)?;

        let db = Database::open(path, true)?;

        let cache_dir = path.join(".cache");
        if cache_dir.exists() {
            tracing::info!("Migrating from JSON cache to SQLite...");
            db.rebuild(path, None, None)?;
            std::fs::remove_dir_all(&cache_dir)?;
            tracing::info!("Migration complete, deleted .cache/");
        }

        Ok(Store {
            root: path.to_path_buf(),
            config,
            db,
        })
    }

    /// Open a store without validation.
    ///
    /// The auto_repair parameter controls whether the database will automatically
    /// repair inconsistencies on open. Set to false for operations like `doctor`
    /// that want to detect issues without fixing them.
    #[tracing::instrument(skip(path), fields(path = %path.display()))]
    pub fn open_unchecked(path: &Path, auto_repair: bool) -> Result<Self> {
        if !path.is_dir() {
            return Err(QipuError::StoreNotFound {
                search_root: path.to_path_buf(),
            });
        }

        let config_path = path.join(CONFIG_FILE);
        let config = if config_path.exists() {
            StoreConfig::load(&config_path)?
        } else {
            // Use default config if missing
            StoreConfig::default()
        };

        let db = Database::open(path, auto_repair)?;

        let cache_dir = path.join(".cache");
        if cache_dir.exists() {
            tracing::info!("Migrating from JSON cache to SQLite...");
            db.rebuild(path, None, None)?;
            std::fs::remove_dir_all(&cache_dir)?;
            tracing::info!("Migration complete, deleted .cache/");
        }

        Ok(Store {
            root: path.to_path_buf(),
            config,
            db,
        })
    }

    /// Initialize a new store under the given project root.
    pub fn init(project_root: &Path, options: InitOptions) -> Result<Self> {
        let store_name = if options.visible {
            VISIBLE_STORE_DIR
        } else {
            DEFAULT_STORE_DIR
        };

        let store_path = project_root.join(store_name);
        Self::init_at(&store_path, options, Some(project_root))
    }

    /// Initialize a store at an explicit store root path.
    pub fn init_at(
        store_root: &Path,
        options: InitOptions,
        project_root: Option<&Path>,
    ) -> Result<Self> {
        // Handle protected branch workflow if requested
        let original_branch = if let Some(branch_name) = &options.branch {
            use crate::git;

            // Verify git is available
            if !git::is_git_available() {
                return Err(QipuError::Other(
                    "Git is required for --branch workflow but was not found in PATH. \
                     Please install git or initialize without --branch."
                        .to_string(),
                ));
            }

            // Determine repository root (use project_root or store parent)
            let repo_root = project_root
                .or_else(|| store_root.parent())
                .ok_or_else(|| {
                    QipuError::Other(
                        "Cannot determine repository root for branch workflow".to_string(),
                    )
                })?;

            // Setup branch workflow (create/switch to branch)
            Some(git::setup_branch_workflow(repo_root, branch_name)?)
        } else {
            None
        };

        // Create directory structure (idempotent)
        fs::create_dir_all(store_root)?;
        fs::create_dir_all(store_root.join(NOTES_DIR))?;
        fs::create_dir_all(store_root.join(MOCS_DIR))?;
        fs::create_dir_all(store_root.join(ATTACHMENTS_DIR))?;
        fs::create_dir_all(store_root.join(TEMPLATES_DIR))?;

        // Create default config if missing (avoid rewriting on subsequent init)
        let config_path = store_root.join(CONFIG_FILE);
        let config_existed = config_path.exists();
        let mut config = if config_existed {
            StoreConfig::load(&config_path)?
        } else {
            StoreConfig::default()
        };

        // Store branch name in config for future operations (if provided)
        if options.branch.is_some() {
            config.branch = options.branch.clone();
        }

        // Save config if it's new or if branch was set
        if !config_existed || options.branch.is_some() {
            config.save(&config_path)?;
        }

        io::ensure_store_gitignore(store_root)?;
        io::ensure_default_templates(&store_root.join(TEMPLATES_DIR))?;

        // Handle stealth mode (add store to project .gitignore)
        if options.stealth {
            if let (Some(project_root), Some(store_name)) = (project_root, store_root.file_name()) {
                if store_root.parent() == Some(project_root) {
                    let project_gitignore = project_root.join(crate::store::paths::GITIGNORE_FILE);
                    let entry = format!("{}/", store_name.to_string_lossy());
                    config::ensure_project_gitignore_entry(&project_gitignore, &entry)?;
                }
            }
        }

        // Switch back to original branch if we were using branch workflow
        if let Some(orig_branch) = original_branch {
            use crate::git;

            let repo_root = project_root
                .or_else(|| store_root.parent())
                .ok_or_else(|| {
                    QipuError::Other(
                        "Cannot determine repository root for branch checkout".to_string(),
                    )
                })?;

            git::checkout_branch(repo_root, &orig_branch)?;
        }

        // Open database without auto-indexing to check config
        let db = Database::open(store_root, true)?;

        // Perform adaptive indexing if enabled
        if !options.no_index && config.auto_index.enabled {
            use crate::db::indexing::IndexingStrategy;

            let force_strategy = options
                .index_strategy
                .as_deref()
                .and_then(IndexingStrategy::parse);
            let _ = db.adaptive_index(store_root, &config.auto_index, force_strategy);
        }

        Ok(Store {
            root: store_root.to_path_buf(),
            config,
            db,
        })
    }

    /// Get the store root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the notes directory
    pub fn notes_dir(&self) -> PathBuf {
        self.root.join(NOTES_DIR)
    }

    /// Get the MOCs directory
    pub fn mocs_dir(&self) -> PathBuf {
        self.root.join(MOCS_DIR)
    }

    /// Get the attachments directory
    pub fn attachments_dir(&self) -> PathBuf {
        self.root.join(ATTACHMENTS_DIR)
    }

    /// Get the templates directory
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join(TEMPLATES_DIR)
    }

    /// Get the workspaces directory
    pub fn workspaces_dir(&self) -> PathBuf {
        self.root.join(WORKSPACES_DIR)
    }

    /// Get the config file path
    pub fn config_path(&self) -> PathBuf {
        self.root.join(CONFIG_FILE)
    }

    /// Get the database file path
    pub fn db_path(&self) -> PathBuf {
        self.root.join("qipu.db")
    }

    /// Get the config
    pub fn config(&self) -> &StoreConfig {
        &self.config
    }

    /// Get the database
    pub fn db(&self) -> &Database {
        &self.db
    }
}

#[cfg(test)]
mod tests;
