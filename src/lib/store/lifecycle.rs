//! Note creation and lifecycle management

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::lib::error::{QipuError, Result};
use crate::lib::id::{filename, NoteId};
use crate::lib::note::{Note, NoteFrontmatter, NoteType};

use super::notes;
use super::paths::{MOCS_DIR, NOTES_DIR, TEMPLATES_DIR};
use super::Store;

impl Store {
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
            NoteType::Moc => self.root.join(MOCS_DIR),
            _ => self.root.join(NOTES_DIR),
        };

        // Write note to disk
        let file_name = filename(&id, title);
        let file_path = target_dir.join(&file_name);

        let content = note.to_markdown()?;
        fs::write(&file_path, content)?;

        let mut note = note;
        note.path = Some(file_path);

        self.db.insert_note(&note)?;
        let ids = self.existing_ids().unwrap_or_default();
        self.db.insert_edges(&note, &ids)?;

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
            NoteType::Moc => self.root.join(MOCS_DIR),
            _ => self.root.join(NOTES_DIR),
        };

        // Write note to disk
        let file_name = filename(&id, title);
        let file_path = target_dir.join(&file_name);

        let markdown = note.to_markdown()?;
        fs::write(&file_path, markdown)?;

        let mut note = note;
        note.path = Some(file_path);

        self.db.insert_note(&note)?;
        let ids = self.existing_ids().unwrap_or_default();
        self.db.insert_edges(&note, &ids)?;

        Ok(note)
    }

    /// Load a template for a note type
    pub(super) fn load_template(&self, note_type: NoteType) -> Result<String> {
        let template_name = format!("{}.md", note_type);
        let template_path = self.root.join(TEMPLATES_DIR).join(&template_name);

        if template_path.exists() {
            // Read template and strip any frontmatter
            let content = fs::read_to_string(&template_path)?;
            Ok(notes::strip_frontmatter(&content))
        } else {
            // Return default body based on type
            Ok(notes::default_body(note_type))
        }
    }

    /// Check if a note with a given ID exists in the store
    pub fn note_exists(&self, id: &str) -> bool {
        match self.existing_ids() {
            Ok(ids) => ids.contains(id),
            Err(_) => false,
        }
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

            // Update database after file write to maintain consistency
            self.db.insert_note(note)?;
            let ids = self.existing_ids().unwrap_or_default();
            self.db.insert_edges(note, &ids)?;
        }

        Ok(())
    }

    /// Get all existing note IDs in the store
    ///
    /// Returns IDs from:
    /// 1. Current database (notes in current working tree)
    /// 2. All git branches (if in a git repository)
    ///
    /// This provides collision avoidance for multi-branch workflows.
    pub fn existing_ids(&self) -> Result<HashSet<String>> {
        // Start with IDs from current database
        let mut ids: HashSet<String> = self.db.list_note_ids()?.into_iter().collect();

        // Add IDs from all git branches if we're in a git repo
        if let Some(repo_root) = self.find_repo_root() {
            // Determine store subpath relative to repo root
            let store_subpath = if let Ok(rel_path) = self.root.strip_prefix(&repo_root) {
                format!("{}/", rel_path.display())
            } else {
                // Store is not under repo root, use empty subpath
                String::new()
            };

            if let Ok(git_ids) =
                crate::lib::git::get_ids_from_all_branches(&repo_root, &store_subpath)
            {
                ids.extend(git_ids);
            }
        }

        Ok(ids)
    }

    /// Find the git repository root (if any) for this store
    fn find_repo_root(&self) -> Option<PathBuf> {
        let mut current = self.root.as_path();

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Some(current.to_path_buf());
            }

            current = current.parent()?;
        }
    }

    #[allow(dead_code)]
    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        let note = self.get_note(note_id)?;
        let path = note
            .path
            .as_ref()
            .ok_or_else(|| QipuError::Other("note has no path".to_string()))?;

        fs::remove_file(path).map_err(|e| {
            QipuError::Other(format!(
                "failed to delete note file {}: {}",
                path.display(),
                e
            ))
        })?;

        self.db.delete_note(note_id)?;

        Ok(())
    }
}
