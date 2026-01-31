use crate::error::Result;
use crate::note::Note;

use super::insert_helper::{insert_note_with_options, InsertOptions};

impl super::super::Database {
    pub fn insert_note(&self, note: &Note) -> Result<()> {
        insert_note_with_options(&self.conn, note, InsertOptions::Standard)
    }

    pub(crate) fn insert_note_internal(conn: &rusqlite::Connection, note: &Note) -> Result<()> {
        insert_note_with_options(conn, note, InsertOptions::Internal)
    }
}
