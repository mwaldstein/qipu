use crate::lib::note::Note;

/// Estimate note size for compaction metrics
/// Uses summary-sized content (same as records output)
pub fn estimate_size(note: &Note) -> usize {
    // Use summary if present
    if let Some(summary) = &note.frontmatter.summary {
        return summary.len();
    }

    // Otherwise use first paragraph or truncated body
    let summary = note.summary();
    summary.len()
}
