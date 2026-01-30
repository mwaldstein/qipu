//! Provenance field handling helpers

use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;

/// Update provenance fields on a note if any are provided
///
/// This helper consolidates the provenance field handling logic that was
/// duplicated across `create` and `capture` commands.
///
/// # Arguments
/// - `store`: The note store for saving changes
/// - `note`: Mutable reference to the note to update
/// - `source`: Original source URL/reference
/// - `author`: Human or agent who created the note
/// - `generated_by`: LLM model name if AI-generated
/// - `prompt_hash`: Hash/ID of the generation prompt
/// - `verified`: Manual review flag (defaults to false when AI-generated)
/// - `is_capture`: If true, sets author to "Qipu Clipper" when source is provided
///
/// # Returns
/// `Ok(true)` if any provenance fields were updated and saved, `Ok(false)` otherwise
pub fn update_provenance_if_provided(
    store: &Store,
    note: &mut Note,
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
    is_capture: bool,
) -> Result<bool> {
    let has_source = source.is_some();
    let has_generated_by = generated_by.is_some();

    let has_provenance = has_source
        || author.is_some()
        || has_generated_by
        || prompt_hash.is_some()
        || verified.is_some();

    if !has_provenance {
        return Ok(false);
    }

    note.frontmatter.source = source;

    note.frontmatter.author = if author.is_some() {
        author
    } else if is_capture && has_source {
        Some("Qipu Clipper".to_string())
    } else {
        None
    };

    note.frontmatter.generated_by = generated_by;
    note.frontmatter.prompt_hash = prompt_hash;

    note.frontmatter.verified = if verified.is_some() {
        verified
    } else if has_generated_by {
        Some(false)
    } else {
        None
    };

    store.save_note(note)?;
    Ok(true)
}
