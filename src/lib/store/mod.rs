//! Store management for qipu
//!
//! The store is the root directory containing all qipu data.
//! Default location: `.qipu/` (hidden, git-trackable)

pub mod config;
pub mod io;
pub mod notes;
pub mod paths;
pub mod workspace;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::lib::config::StoreConfig;
use crate::lib::error::{QipuError, Result};
use crate::lib::id::{filename, NoteId};
use crate::lib::logging;
use crate::lib::note::{Note, NoteFrontmatter, NoteType};
pub use config::InitOptions;
use paths::{
    ATTACHMENTS_DIR, CACHE_DIR, CONFIG_FILE, DEFAULT_STORE_DIR, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR,
    VISIBLE_STORE_DIR,
};

/// The qipu store
#[derive(Debug)]
pub struct Store {
    /// Root path of the store
    root: PathBuf,
    /// Store configuration
    config: StoreConfig,
}

impl Store {
    /// Discover a store by walking up from the given root directory
    pub fn discover(root: &Path) -> Result<Self> {
        let store_path = paths::discover_store(root)?;
        Self::open(&store_path)
    }

    /// Open an existing store at the given path
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

        Ok(Store {
            root: path.to_path_buf(),
            config,
        })
    }

    /// Open a store without validation.
    pub fn open_unchecked(path: &Path) -> Result<Self> {
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

        Ok(Store {
            root: path.to_path_buf(),
            config,
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
            use crate::lib::git;

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
        fs::create_dir_all(store_root.join(CACHE_DIR))?;

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
                    let project_gitignore =
                        project_root.join(crate::lib::store::paths::GITIGNORE_FILE);
                    let entry = format!("{}/", store_name.to_string_lossy());
                    config::ensure_project_gitignore_entry(&project_gitignore, &entry)?;
                }
            }
        }

        // Switch back to original branch if we were using branch workflow
        if let Some(orig_branch) = original_branch {
            use crate::lib::git;

            let repo_root = project_root
                .or_else(|| store_root.parent())
                .ok_or_else(|| {
                    QipuError::Other(
                        "Cannot determine repository root for branch checkout".to_string(),
                    )
                })?;

            git::checkout_branch(repo_root, &orig_branch)?;
        }

        Ok(Store {
            root: store_root.to_path_buf(),
            config,
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

    /// Get the templates directory
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join(TEMPLATES_DIR)
    }

    /// Get the config
    pub fn config(&self) -> &StoreConfig {
        &self.config
    }

    /// Get all existing note IDs in the store
    pub fn existing_ids(&self) -> Result<HashSet<String>> {
        let mut ids = HashSet::new();

        for dir in [self.notes_dir(), self.mocs_dir()] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|e| e == "md") {
                    // Try to extract ID from filename (format: qp-xxxx-slug.md)
                    if let Some(name) = entry.path().file_stem() {
                        let name = name.to_string_lossy();
                        if let Some(id_end) = name.find('-').and_then(|first| {
                            name[first + 1..].find('-').map(|second| first + 1 + second)
                        }) {
                            ids.insert(name[..id_end].to_string());
                        } else if name.starts_with("qp-") {
                            // Might be just qp-xxxx.md
                            ids.insert(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(ids)
    }

    /// Check if a note with a given ID exists in the store
    pub fn note_exists(&self, id: &str) -> bool {
        match self.existing_ids() {
            Ok(ids) => ids.contains(id),
            Err(_) => false,
        }
    }

    /// Create a new note
    pub fn create_note(
        &self,
        title: &str,
        note_type: Option<NoteType>,
        tags: &[String],
        id: Option<&str>,
    ) -> Result<Note> {
        let id = if let Some(id) = id {
            if self.note_exists(id) {
                return Err(QipuError::Other(format!(
                    "note with id '{}' already exists",
                    id
                )));
            }
            NoteId::new_unchecked(id.to_string())
        } else {
            let existing_ids = self.existing_ids()?;
            NoteId::generate(self.config.id_scheme, title, &existing_ids)
        };

        let note_type = note_type.unwrap_or(self.config.default_note_type);
        let frontmatter = NoteFrontmatter::new(id.to_string(), title.to_string())
            .with_type(note_type)
            .with_tags(tags.iter().cloned());

        // Try to load template
        let body = self.load_template(note_type)?;

        let note = Note::new(frontmatter, body);

        // Determine target directory
        let target_dir = match note_type {
            NoteType::Moc => self.mocs_dir(),
            _ => self.notes_dir(),
        };

        // Write note to disk
        let file_name = filename(&id, title);
        let file_path = target_dir.join(&file_name);

        let content = note.to_markdown()?;
        fs::write(&file_path, content)?;

        let mut note = note;
        note.path = Some(file_path);

        Ok(note)
    }

    /// Create a new note with specific content (used by capture command)
    pub fn create_note_with_content(
        &self,
        title: &str,
        note_type: Option<NoteType>,
        tags: &[String],
        content: &str,
        id: Option<&str>,
    ) -> Result<Note> {
        let existing_ids = self.existing_ids()?;
        let id = if let Some(id) = id {
            if existing_ids.contains(id) {
                return Err(QipuError::Other(format!(
                    "note with id '{}' already exists",
                    id
                )));
            }
            NoteId::new_unchecked(id.to_string())
        } else {
            NoteId::generate(self.config.id_scheme, title, &existing_ids)
        };

        let note_type = note_type.unwrap_or(self.config.default_note_type);
        let frontmatter = NoteFrontmatter::new(id.to_string(), title.to_string())
            .with_type(note_type)
            .with_tags(tags.iter().cloned());

        // Use provided content instead of template
        let note = Note::new(frontmatter, content.to_string());

        // Determine target directory
        let target_dir = match note_type {
            NoteType::Moc => self.mocs_dir(),
            _ => self.notes_dir(),
        };

        // Write note to disk
        let file_name = filename(&id, title);
        let file_path = target_dir.join(&file_name);

        let markdown = note.to_markdown()?;
        fs::write(&file_path, markdown)?;

        let mut note = note;
        note.path = Some(file_path);

        Ok(note)
    }

    /// Load a template for a note type
    fn load_template(&self, note_type: NoteType) -> Result<String> {
        let template_name = format!("{}.md", note_type);
        let template_path = self.templates_dir().join(&template_name);

        if template_path.exists() {
            // Read template and strip any frontmatter
            let content = fs::read_to_string(&template_path)?;
            Ok(notes::strip_frontmatter(&content))
        } else {
            // Return default body based on type
            Ok(notes::default_body(note_type))
        }
    }

    /// List all notes in the store
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();

        for dir in [self.notes_dir(), self.mocs_dir()] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    match Note::parse(&fs::read_to_string(path)?, Some(path.to_path_buf())) {
                        Ok(note) => notes.push(note),
                        Err(e) => {
                            // Log but continue - don't fail on individual bad notes
                            if logging::verbose_enabled() {
                                eprintln!("Warning: failed to parse {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Sort by created date (newest first), then by id for stability
        notes.sort_by(|a, b| {
            match (&b.frontmatter.created, &a.frontmatter.created) {
                (Some(b_created), Some(a_created)) => b_created.cmp(a_created),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then_with(|| a.id().cmp(b.id()))
        });

        Ok(notes)
    }

    /// Get a note by ID
    pub fn get_note(&self, id: &str) -> Result<Note> {
        self.get_note_internal(id, None)
    }

    /// Get a note by ID using an index for fast path lookup
    pub fn get_note_with_index(&self, id: &str, index: &crate::lib::index::Index) -> Result<Note> {
        // Try fast lookup using index first
        if let Some(path) = index.get_note_path(id) {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                return Note::parse(&content, Some(path.clone()));
            }
        }

        // Fallback to directory traversal
        self.get_note_internal(id, Some(index))
    }

    /// Internal note lookup implementation
    fn get_note_internal(
        &self,
        id: &str,
        index: Option<&crate::lib::index::Index>,
    ) -> Result<Note> {
        // If we have an index, try to use its path information
        if let Some(idx) = index {
            if let Some(path) = idx.get_note_path(id) {
                if path.exists() {
                    let content = fs::read_to_string(path)?;
                    return Note::parse(&content, Some(path.clone()));
                }
            }
        }

        // Search in both notes and mocs directories
        for dir in [self.notes_dir(), self.mocs_dir()] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    // Check if filename starts with the ID
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy();
                        if name.starts_with(id)
                            && (name.len() == id.len() || name.chars().nth(id.len()) == Some('-'))
                        {
                            let content = fs::read_to_string(path)?;
                            return Note::parse(&content, Some(path.to_path_buf()));
                        }
                    }
                }
            }
        }

        Err(QipuError::NoteNotFound { id: id.to_string() })
    }

    /// Save an existing note back to disk
    pub fn save_note(&self, note: &mut Note) -> Result<()> {
        let path = note
            .path
            .as_ref()
            .ok_or_else(|| QipuError::Other("cannot save note without path".to_string()))?;

        // Auto-populate the updated timestamp
        note.frontmatter.updated = Some(chrono::Utc::now());

        let new_content = note.to_markdown()?;

        // Filesystem hygiene: only write if content actually changed
        let should_write = if path.exists() {
            match fs::read_to_string(path) {
                Ok(existing) => existing != new_content,
                Err(_) => true,
            }
        } else {
            true
        };

        if should_write {
            fs::write(path, new_content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::config::STORE_FORMAT_VERSION;
    use crate::lib::id::IdScheme;
    use tempfile::tempdir;

    #[test]
    fn test_init_store() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        assert!(store.root().exists());
        assert!(store.notes_dir().exists());
        assert!(store.mocs_dir().exists());
        assert!(store.templates_dir().exists());
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
        assert_eq!(store.config().default_note_type, NoteType::Fleeting);
        assert_eq!(store.config().id_scheme, IdScheme::Hash);

        let note = store.create_note("Test Note", None, &[], None).unwrap();
        assert!(note.id().starts_with("qp-"));

        assert!(store.templates_dir().join("fleeting.md").exists());
        assert!(store.templates_dir().join("permanent.md").exists());
    }
}
