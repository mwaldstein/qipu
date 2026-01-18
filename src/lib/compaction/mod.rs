//! Compaction utilities for qipu
//!
//! Implements digest-first navigation and lossless knowledge decay.
//! Per spec: specs/compaction.md

mod context;
mod expansion;
mod suggestion;
mod validation;

pub use context::CompactionContext;

/// Estimate note size for compaction metrics
/// Uses summary-sized content (same as records output)
/// Per spec: specs/compaction.md lines 168-175
pub(crate) fn estimate_note_size(note: &crate::lib::note::Note) -> usize {
    // Use summary if present in frontmatter
    if let Some(summary) = &note.frontmatter.summary {
        return summary.len();
    }

    // Otherwise use first paragraph or truncated body
    let summary = note.summary();
    summary.len()
}
