//! Store management for qipu
//!
//! The store is the root directory containing all qipu data.
//! Default location: `.qipu/` (hidden, git-trackable)
//!
//! Per spec (specs/storage-format.md):
//! ```
//! .qipu/
//!   config.toml           # Store configuration
//!   notes/                # All non-MOC notes
//!   mocs/                 # Map of content notes
//!   attachments/          # Optional binaries (images, PDFs)
//!   templates/            # Note templates
//!   qipu.db               # Optional derived local index (gitignored)
//!   .cache/               # Derived; safe to delete
//! ```

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::lib::config::StoreConfig;
use crate::lib::error::{QipuError, Result};
use crate::lib::id::{filename, NoteId};
use crate::lib::note::{Note, NoteFrontmatter, NoteType};

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

/// Gitignore filename
pub const GITIGNORE_FILE: &str = ".gitignore";

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
    ///
    /// Per spec (specs/cli-tool.md):
    /// 1. If `--store` is provided, use it
    /// 2. Otherwise, walk up from `--root` (or cwd) looking for `.qipu/`
    /// 3. If filesystem root reached, store is "missing"
    pub fn discover(root: &Path) -> Result<Self> {
        let mut current = root.to_path_buf();

        loop {
            // Check for default hidden store
            let store_path = current.join(DEFAULT_STORE_DIR);
            if store_path.is_dir() {
                return Self::open(&store_path);
            }

            // Check for visible store
            let visible_path = current.join(VISIBLE_STORE_DIR);
            if visible_path.is_dir() {
                return Self::open(&visible_path);
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

    /// Open an existing store at the given path
    pub fn open(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            return Err(QipuError::StoreNotFound {
                search_root: path.to_path_buf(),
            });
        }

        let config_path = path.join(CONFIG_FILE);
        let config = if config_path.exists() {
            StoreConfig::load(&config_path)?
        } else {
            StoreConfig::default()
        };

        Ok(Store {
            root: path.to_path_buf(),
            config,
        })
    }

    /// Initialize a new store
    pub fn init(root: &Path, options: InitOptions) -> Result<Self> {
        let store_name = if options.visible {
            VISIBLE_STORE_DIR
        } else {
            DEFAULT_STORE_DIR
        };

        let store_path = root.join(store_name);

        // Create store directory (idempotent)
        if store_path.exists() {
            // Already exists - open it instead
            return Self::open(&store_path);
        }

        // Create directory structure
        fs::create_dir_all(&store_path)?;
        fs::create_dir_all(store_path.join(NOTES_DIR))?;
        fs::create_dir_all(store_path.join(MOCS_DIR))?;
        fs::create_dir_all(store_path.join(ATTACHMENTS_DIR))?;
        fs::create_dir_all(store_path.join(TEMPLATES_DIR))?;
        fs::create_dir_all(store_path.join(CACHE_DIR))?;

        // Create default config
        let config = StoreConfig::default();
        config.save(&store_path.join(CONFIG_FILE))?;

        // Create .gitignore for store
        let gitignore_content = "qipu.db\n.cache/\n";
        fs::write(store_path.join(GITIGNORE_FILE), gitignore_content)?;

        // Create default templates
        create_default_templates(&store_path.join(TEMPLATES_DIR))?;

        // Handle stealth mode (add to project .gitignore)
        if options.stealth {
            let project_gitignore = root.join(GITIGNORE_FILE);
            let entry = format!("{}/\n", store_name);

            if project_gitignore.exists() {
                let content = fs::read_to_string(&project_gitignore)?;
                if !content.contains(&entry) {
                    fs::write(&project_gitignore, format!("{}{}", content, entry))?;
                }
            } else {
                fs::write(&project_gitignore, entry)?;
            }
        }

        Ok(Store {
            root: store_path,
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
                if entry.path().extension().map_or(false, |e| e == "md") {
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

    /// Create a new note
    pub fn create_note(
        &self,
        title: &str,
        note_type: Option<NoteType>,
        tags: &[String],
    ) -> Result<Note> {
        let existing_ids = self.existing_ids()?;
        let id = NoteId::generate(self.config.id_scheme, title, &existing_ids);

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
    ) -> Result<Note> {
        let existing_ids = self.existing_ids()?;
        let id = NoteId::generate(self.config.id_scheme, title, &existing_ids);

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
            Ok(strip_frontmatter(&content))
        } else {
            // Return default body based on type
            Ok(default_body(note_type))
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
                if path.extension().map_or(false, |e| e == "md") {
                    match Note::parse(&fs::read_to_string(path)?, Some(path.to_path_buf())) {
                        Ok(note) => notes.push(note),
                        Err(e) => {
                            // Log but continue - don't fail on individual bad notes
                            eprintln!("Warning: failed to parse {}: {}", path.display(), e);
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
                if path.extension().map_or(false, |e| e == "md") {
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
}

/// Options for store initialization
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// Use visible store directory (`qipu/` instead of `.qipu/`)
    pub visible: bool,
    /// Stealth mode (add store to .gitignore)
    pub stealth: bool,
    /// Branch workflow (not yet implemented)
    #[allow(dead_code)]
    pub branch: Option<String>,
}

/// Create default templates for each note type
fn create_default_templates(templates_dir: &Path) -> Result<()> {
    // Fleeting template
    let fleeting = r#"## Summary

<!-- One-sentence summary of this thought -->

## Notes

<!-- Quick capture - refine later -->
"#;

    // Literature template
    let literature = r#"## Summary

<!-- Key takeaway from this source -->

## Notes

<!-- Your notes on this external source -->

## Quotes

<!-- Notable quotes from the source -->
"#;

    // Permanent template
    let permanent = r#"## Summary

<!-- One idea, in your own words, that can stand alone -->

## Notes

<!-- Explanation and context -->

## See Also

<!-- Related notes: explain *why* each is related, not just bare links -->
"#;

    // MOC template
    let moc = r#"## Summary

<!-- What this map covers and why it exists -->

## Overview

<!-- Brief introduction to the topic -->

## Reading Path

<!-- Suggested order for exploring this topic -->

## Topics

<!-- Organized links to notes, grouped by subtopic -->
<!-- Explain what belongs here and why -->
"#;

    fs::write(templates_dir.join("fleeting.md"), fleeting)?;
    fs::write(templates_dir.join("literature.md"), literature)?;
    fs::write(templates_dir.join("permanent.md"), permanent)?;
    fs::write(templates_dir.join("moc.md"), moc)?;

    Ok(())
}

/// Strip frontmatter from template content
fn strip_frontmatter(content: &str) -> String {
    let content = content.trim_start();
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            let after_fm = &content[3 + end + 4..];
            return after_fm.trim_start_matches('\n').to_string();
        }
    }
    content.to_string()
}

/// Get default body for a note type
fn default_body(note_type: NoteType) -> String {
    match note_type {
        NoteType::Fleeting => "## Summary\n\n\n\n## Notes\n\n".to_string(),
        NoteType::Literature => "## Summary\n\n\n\n## Notes\n\n\n\n## Quotes\n\n".to_string(),
        NoteType::Permanent => "## Summary\n\n\n\n## Notes\n\n\n\n## See Also\n\n".to_string(),
        NoteType::Moc => {
            "## Summary\n\n\n\n## Overview\n\n\n\n## Reading Path\n\n\n\n## Topics\n\n".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let store = Store::init(dir.path(), options).unwrap();

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

        let note = store.create_note("Test Note", None, &[]).unwrap();
        assert!(note.id().starts_with("qp-"));
        assert_eq!(note.title(), "Test Note");
        assert!(note.path.is_some());
        assert!(note.path.as_ref().unwrap().exists());
    }

    #[test]
    fn test_list_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Note 1", None, &[]).unwrap();
        store.create_note("Note 2", None, &[]).unwrap();

        let notes = store.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }
}
