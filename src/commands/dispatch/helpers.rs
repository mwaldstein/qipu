//! Helper functions for command dispatch

use std::path::Path;

use crate::cli::Cli;
use qipu_core::error::Result;
use qipu_core::note::Note;

use super::command::discover_or_open_store;

/// Resolve a note by ID or path.
///
/// If `id_or_path` refers to an existing file, reads and parses it.
/// Otherwise, treats it as a note ID and looks it up in the store.
pub fn resolve_note_by_id_or_path(cli: &Cli, root: &Path, id_or_path: &str) -> Result<Note> {
    let store = discover_or_open_store(cli, root)?;
    store.load_note_by_id_or_path(id_or_path)
}
