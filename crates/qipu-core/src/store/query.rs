//! Note query and retrieval operations

use crate::error::{QipuError, Result};
use crate::note::Note;

use super::Store;

impl Store {
    /// List all notes in the store
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let db = self.db();
        let mut notes = db.list_notes_full()?;

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
    #[tracing::instrument(skip(self), fields(note_id = %id))]
    pub fn get_note(&self, id: &str) -> Result<Note> {
        self.get_note_internal(id)
    }

    /// Internal note lookup implementation
    pub(super) fn get_note_internal(&self, id: &str) -> Result<Note> {
        let db = self.db();
        db.get_note(id)?
            .ok_or_else(|| QipuError::NoteNotFound { id: id.to_string() })
    }

    /// Get tag frequency statistics
    pub fn get_tag_frequencies(&self) -> Result<Vec<(String, i64)>> {
        let db = self.db();
        db.get_tag_frequencies()
    }
}
