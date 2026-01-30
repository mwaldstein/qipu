//! Helper functions shared across commands

use std::env;

/// Resolve editor to use from override, EDITOR, VISUAL, or fallback
///
/// Returns None if no editor is configured
pub fn resolve_editor(editor_override: Option<&str>) -> Option<String> {
    editor_override
        .map(String::from)
        .or_else(|| env::var("EDITOR").ok())
        .or_else(|| env::var("VISUAL").ok())
}
