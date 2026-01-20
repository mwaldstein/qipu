//! Note query and retrieval operations

use std::fs;

use crate::lib::error::{QipuError, Result};
use crate::lib::note::Note;

use super::Store;

impl Store {
    /// List all notes in the store
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let db = self.db();
        let metadatas = db.list_notes(None, None, None)?;

        let mut notes = Vec::new();
        for metadata in metadatas {
            let path = self.root.join(&metadata.path);
            match Note::parse(&fs::read_to_string(&path)?, Some(path)) {
                Ok(note) => notes.push(note),
                Err(e) => {
                    tracing::warn!(path = %metadata.path, error = %e, "Failed to parse note");
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
        self.get_note_internal(id)
    }

    /// Get a note by ID using an index for fast path lookup
    #[allow(dead_code)]
    pub fn get_note_with_index(&self, id: &str, _index: &crate::lib::index::Index) -> Result<Note> {
        self.get_note_internal(id)
    }

    /// Internal note lookup implementation
    pub(super) fn get_note_internal(&self, id: &str) -> Result<Note> {
        let db = self.db();
        let meta = db
            .get_note_metadata(id)?
            .ok_or_else(|| QipuError::NoteNotFound { id: id.to_string() })?;

        let path = self.root().join(&meta.path);
        let content = fs::read_to_string(&path).map_err(|e| {
            tracing::warn!(path = %meta.path, error = %e, "Failed to read note file");
            QipuError::NoteNotFound { id: id.to_string() }
        })?;

        Note::parse(&content, Some(path)).map_err(|e| {
            tracing::warn!(path = %meta.path, error = %e, "Failed to parse note");
            QipuError::NoteNotFound { id: id.to_string() }
        })
    }
}
