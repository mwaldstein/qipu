//! Provenance field handling helpers

use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;

/// Optional provenance fields for note updates
pub struct ProvenanceUpdate<'a> {
    pub source: Option<&'a str>,
    pub author: Option<&'a str>,
    pub generated_by: Option<&'a str>,
    pub prompt_hash: Option<&'a str>,
    pub verified: Option<bool>,
}

/// Update provenance fields on a note if any are provided
///
/// This helper consolidates the provenance field handling logic that was
/// duplicated across `create` and `capture` commands.
///
/// # Arguments
/// - `store`: The note store for saving changes
/// - `note`: Mutable reference to the note to update
/// - `update`: Optional provenance fields to apply
/// - `is_capture`: If true, sets author to "Qipu Clipper" when source is provided
///
/// # Returns
/// `Ok(true)` if any provenance fields were updated and saved, `Ok(false)` otherwise
pub fn update_provenance_if_provided<'a>(
    store: &Store,
    note: &mut Note,
    update: ProvenanceUpdate<'a>,
    is_capture: bool,
) -> Result<bool> {
    let has_source = update.source.is_some();
    let has_generated_by = update.generated_by.is_some();

    let has_provenance = has_source
        || update.author.is_some()
        || has_generated_by
        || update.prompt_hash.is_some()
        || update.verified.is_some();

    if !has_provenance {
        return Ok(false);
    }

    note.frontmatter.source = update.source.map(|s| s.to_string());

    note.frontmatter.author = if update.author.is_some() {
        update.author.map(|s| s.to_string())
    } else if is_capture && has_source {
        Some("Qipu Clipper".to_string())
    } else {
        None
    };

    note.frontmatter.generated_by = update.generated_by.map(|s| s.to_string());
    note.frontmatter.prompt_hash = update.prompt_hash.map(|s| s.to_string());

    note.frontmatter.verified = if update.verified.is_some() {
        update.verified
    } else if has_generated_by {
        Some(false)
    } else {
        None
    };

    store.save_note(note)?;
    Ok(true)
}
