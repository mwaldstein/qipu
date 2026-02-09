//! Helper functions shared across commands

use std::collections::HashSet;
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

/// Extract attachment filenames referenced in note body
///
/// Looks for markdown image/link patterns referencing ../attachments/
/// Returns a set of unique attachment filenames
pub fn extract_attachment_references(body: &str) -> HashSet<String> {
    let mut attachments = HashSet::new();

    // Pattern to match ![alt](../attachments/filename.ext) or [text](../attachments/filename.ext)
    for line in body.lines() {
        let mut search_start = 0;
        while let Some(start) = line[search_start..].find("(") {
            let actual_start = search_start + start;
            if let Some(end) = line[actual_start..].find(")") {
                let path_str = &line[actual_start + 1..actual_start + end];
                // Check for attachments path
                if path_str.starts_with("../attachments/") || path_str.starts_with("./attachments/")
                {
                    let filename = path_str
                        .trim_start_matches("../attachments/")
                        .trim_start_matches("./attachments/");
                    if !filename.is_empty() && !filename.contains('/') {
                        attachments.insert(filename.to_string());
                    }
                }
                search_start = actual_start + end + 1;
            } else {
                break;
            }
        }
    }

    attachments
}

/// Copy attachments from source store to destination store
///
/// Given a note and source/destination stores, copies any referenced attachments
/// that exist in the source to the destination.
pub fn copy_note_attachments(
    note_body: &str,
    src_store: &qipu_core::store::Store,
    dst_store: &qipu_core::store::Store,
) -> qipu_core::error::Result<()> {
    let attachments = extract_attachment_references(note_body);

    if attachments.is_empty() {
        return Ok(());
    }

    let src_attachments_dir = src_store.attachments_dir();
    let dst_attachments_dir = dst_store.attachments_dir();

    // Ensure destination attachments directory exists
    if !dst_attachments_dir.exists() {
        std::fs::create_dir_all(&dst_attachments_dir)?;
    }

    for filename in attachments {
        let src_path = src_attachments_dir.join(&filename);
        if src_path.exists() {
            let dst_path = dst_attachments_dir.join(&filename);
            // Only copy if not already present (avoid overwriting)
            if !dst_path.exists() {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}
