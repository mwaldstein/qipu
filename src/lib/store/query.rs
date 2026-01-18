//! Note query and retrieval operations

use std::fs;

use walkdir::WalkDir;

use crate::lib::error::{QipuError, Result};
use crate::lib::logging;
use crate::lib::note::Note;

use super::paths::{MOCS_DIR, NOTES_DIR};
use super::Store;

impl Store {
    /// List all notes in the store
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();

        for dir in [self.root.join(NOTES_DIR), self.root.join(MOCS_DIR)] {
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
    pub(super) fn get_note_internal(
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
        for dir in [self.root.join(NOTES_DIR), self.root.join(MOCS_DIR)] {
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
}
