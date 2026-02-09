//! Interactive picker utilities for fzf-style selection
//!
//! Provides interactive selection of notes using the inquire crate.
//! Used when --interactive flag is passed to list/search/inbox commands.

use inquire::{InquireError, Select};
use qipu_core::error::Result;
use qipu_core::index::SearchResult;
use qipu_core::note::Note;

/// An item that can be displayed in the picker
pub struct PickerItem {
    pub id: String,
    pub display: String,
    #[allow(dead_code)]
    pub note_type: String,
}

impl PickerItem {
    /// Create a picker item from a note
    pub fn from_note(note: &Note) -> Self {
        let type_indicator = match note.note_type().as_str() {
            "fleeting" => "F",
            "literature" => "L",
            "permanent" => "P",
            "moc" => "M",
            _ => "?",
        };

        Self {
            id: note.id().to_string(),
            display: format!("[{}] {}", type_indicator, note.title()),
            note_type: note.note_type().to_string(),
        }
    }

    /// Create a picker item from a search result
    pub fn from_search_result(result: &SearchResult) -> Self {
        let type_indicator = match result.note_type.as_str() {
            "fleeting" => "F",
            "literature" => "L",
            "permanent" => "P",
            "moc" => "M",
            _ => "?",
        };

        Self {
            id: result.id.clone(),
            display: format!("[{}] {}", type_indicator, result.title),
            note_type: result.note_type.to_string(),
        }
    }
}

/// Present an interactive picker for selecting a single note
///
/// Returns the selected note ID, or None if user cancelled/escaped.
pub fn pick_single(items: &[PickerItem], prompt: &str) -> Result<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    let options: Vec<String> = items.iter().map(|item| item.display.clone()).collect();

    let help_message = format!(
        "{} items, use ↑↓ to navigate, Enter to select, Esc to cancel",
        items.len()
    );

    let select = Select::new(prompt, options)
        .with_help_message(&help_message)
        .with_page_size(20);

    match select.prompt() {
        Ok(selection) => {
            // Find the item that matches this display
            let selected = items
                .iter()
                .find(|item| item.display == selection)
                .map(|item| item.id.clone());
            Ok(selected)
        }
        Err(InquireError::OperationCanceled) => Ok(None),
        Err(InquireError::OperationInterrupted) => Ok(None),
        Err(e) => Err(qipu_core::error::QipuError::io_operation(
            "interactive picker",
            "stdin",
            e,
        )),
    }
}

/// Check if stdin is a TTY (interactive terminal)
///
/// Returns false if running in a non-interactive environment (CI, pipe, etc.)
#[allow(dead_code)]
pub fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdin)
}
