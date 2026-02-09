//! Compaction utilities for qipu
//!
//! Implements digest-first navigation and lossless knowledge decay.
//! Per spec: specs/compaction.md

mod context;
mod expansion;
mod suggestion;
mod validation;

pub use context::CompactionContext;
pub use suggestion::CompactionCandidate;

/// Size basis for note size estimation
/// Per spec: specs/compaction.md lines 168-175
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeBasis {
    /// Summary-sized estimate (default, aligned with LLM retrieval)
    /// Uses frontmatter summary or first paragraph
    #[default]
    Summary,
    /// Full body size estimate
    /// Uses complete note body content
    Body,
}

impl SizeBasis {
    /// Parse size basis from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "summary" => Some(SizeBasis::Summary),
            "body" => Some(SizeBasis::Body),
            _ => None,
        }
    }
}

impl std::fmt::Display for SizeBasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeBasis::Summary => write!(f, "summary"),
            SizeBasis::Body => write!(f, "body"),
        }
    }
}

/// Estimate note size for compaction metrics
/// Supports alternate size bases per spec: specs/compaction.md lines 168-175
/// Default is summary-sized for stability
pub(crate) fn estimate_note_size(note: &crate::note::Note, basis: SizeBasis) -> usize {
    match basis {
        SizeBasis::Summary => {
            // Use summary if present in frontmatter
            if let Some(summary) = &note.frontmatter.summary {
                return summary.len();
            }
            // Otherwise use first paragraph or truncated body
            let summary = note.summary();
            summary.len()
        }
        SizeBasis::Body => {
            // Use full body length
            note.body.len()
        }
    }
}
