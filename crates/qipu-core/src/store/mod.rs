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
pub mod workspace;

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::StoreConfig;
use crate::db::Database;
use crate::error::{QipuError, Result};
pub use config::InitOptions;
use paths::{
    ATTACHMENTS_DIR, CONFIG_FILE, DEFAULT_STORE_DIR, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR,
    VISIBLE_STORE_DIR,
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
mod tests {
    use super::*;
    use crate::config::STORE_FORMAT_VERSION;
    use crate::id::IdScheme;
    use crate::note::NoteType;
    use tempfile::tempdir;

    #[test]
    fn test_init_store() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        assert!(store.root().exists());
        assert!(store.notes_dir().exists());
        assert!(store.mocs_dir().exists());
        assert!(store.root().join(CONFIG_FILE).exists());
    }

    #[test]
    fn test_init_visible() {
        let dir = tempdir().unwrap();
        let options = InitOptions {
            visible: true,
            ..Default::default()
        };
        let _store = Store::init(dir.path(), options).unwrap();

        assert!(dir.path().join(VISIBLE_STORE_DIR).exists());
    }

    #[test]
    fn test_discover_store() {
        let dir = tempdir().unwrap();
        Store::init(dir.path(), InitOptions::default()).unwrap();

        let discovered = Store::discover(dir.path()).unwrap();
        assert_eq!(discovered.root(), dir.path().join(DEFAULT_STORE_DIR));
    }

    #[test]
    fn test_create_note() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store.create_note("Test Note", None, &[], None).unwrap();
        assert!(note.id().starts_with("qp-"));
        assert_eq!(note.title(), "Test Note");
        assert!(note.path.is_some());
        assert!(note.path.as_ref().unwrap().exists());
    }

    #[test]
    fn test_list_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Note 1", None, &[], None).unwrap();
        store.create_note("Note 2", None, &[], None).unwrap();

        let notes = store.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_store_without_config() {
        let dir = tempdir().unwrap();
        let store_root = dir.path().join(DEFAULT_STORE_DIR);

        fs::create_dir_all(store_root.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(store_root.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(store_root.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(store_root.join(TEMPLATES_DIR)).unwrap();

        let store = Store::open(&store_root).unwrap();
        assert_eq!(store.config().version, STORE_FORMAT_VERSION);
        assert_eq!(
            store.config().default_note_type,
            NoteType::from(NoteType::FLEETING)
        );
        assert_eq!(store.config().id_scheme, IdScheme::Hash);

        let note = store.create_note("Test Note", None, &[], None).unwrap();
        assert!(note.id().starts_with("qp-"));

        let templates_dir = store.root().join(TEMPLATES_DIR);
        assert!(templates_dir.join("fleeting.md").exists());
        assert!(templates_dir.join("permanent.md").exists());
    }

    #[test]
    fn test_discover_with_custom_store_path() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let default_store = project_root.join(DEFAULT_STORE_DIR);
        let custom_store = project_root.join("custom_notes");

        fs::create_dir_all(&default_store).unwrap();
        fs::create_dir_all(custom_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(TEMPLATES_DIR)).unwrap();

        let config_path = default_store.join(CONFIG_FILE);
        let config = StoreConfig {
            store_path: Some("custom_notes".to_string()),
            ..Default::default()
        };
        config.save(&config_path).unwrap();

        let loaded_config = StoreConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.store_path, Some("custom_notes".to_string()));

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, custom_store);
    }

    #[test]
    fn test_discover_without_custom_store_path() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let default_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(default_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(default_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(default_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(default_store.join(TEMPLATES_DIR)).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, default_store);
    }

    #[test]
    fn test_discovery_stops_at_project_root() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let project_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(project_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(project_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(TEMPLATES_DIR)).unwrap();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join(".git")).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, project_store);

        let result = paths::discover_store(parent_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_store);
    }

    #[test]
    fn test_discovery_fails_at_project_root_without_store() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join(".git")).unwrap();

        let result = paths::discover_store(project_root);
        assert!(result.is_err());
        assert!(matches!(result, Err(QipuError::StoreNotFound { .. })));
    }

    #[test]
    fn test_discovery_stops_at_cargo_toml() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let project_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(project_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(project_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(TEMPLATES_DIR)).unwrap();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join("Cargo.toml")).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, project_store);

        let result = paths::discover_store(parent_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_store);
    }
}
