//! Note creation and lifecycle management

use std::collections::HashSet;
use std::fs;

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
        self.db.insert_edges(&note)?;

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
        self.db.insert_edges(&note)?;

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
        }

        Ok(())
    }

    /// Get all existing note IDs in the store
    pub fn existing_ids(&self) -> Result<HashSet<String>> {
        use walkdir::WalkDir;

        let mut ids = HashSet::new();

        for dir in [self.root.join(NOTES_DIR), self.root.join(MOCS_DIR)] {
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
}
