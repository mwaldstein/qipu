//! Note filtering utilities

use crate::compaction::CompactionContext;
use crate::note::NoteType;
use crate::query::custom_filter::matches_custom_filter;
use chrono::{DateTime, Utc};

/// Filter configuration for notes
#[derive(Debug, Clone)]
pub struct NoteFilter<'a> {
    /// Filter by tag
    pub tag: Option<&'a str>,
    /// Filter by equivalent tags (for alias resolution)
    pub equivalent_tags: Option<Vec<String>>,
    /// Filter by note type
    pub note_type: Option<NoteType>,
    /// Filter by creation date (notes created since this timestamp)
    pub since: Option<DateTime<Utc>>,
    /// Filter by minimum value (0-100, notes with value >= min_value)
    pub min_value: Option<u8>,
    /// Filter by custom metadata
    ///
    /// Supported formats:
    /// - Equality: `key=value`
    /// - Existence: `key` (present), `!key` (absent)
    /// - Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
    pub custom: Option<&'a str>,
    /// Whether to hide compacted notes
    pub hide_compacted: bool,
}

impl<'a> Default for NoteFilter<'a> {
    fn default() -> Self {
        Self {
            tag: None,
            equivalent_tags: None,
            note_type: None,
            since: None,
            min_value: None,
            custom: None,
            hide_compacted: true,
        }
    }
}

impl<'a> NoteFilter<'a> {
    /// Create a new filter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tag filter
    pub fn with_tag(mut self, tag: Option<&'a str>) -> Self {
        self.tag = tag;
        self
    }

    /// Set the equivalent tags (for tag alias resolution)
    pub fn with_equivalent_tags(mut self, tags: Option<Vec<String>>) -> Self {
        self.equivalent_tags = tags;
        self
    }

    /// Set the note type filter
    pub fn with_type(mut self, note_type: Option<NoteType>) -> Self {
        self.note_type = note_type;
        self
    }

    /// Set the since filter
    pub fn with_since(mut self, since: Option<DateTime<Utc>>) -> Self {
        self.since = since;
        self
    }

    /// Set the min-value filter
    pub fn with_min_value(mut self, min_value: Option<u8>) -> Self {
        self.min_value = min_value;
        self
    }

    /// Set the custom metadata filter
    pub fn with_custom(mut self, custom: Option<&'a str>) -> Self {
        self.custom = custom;
        self
    }

    /// Set whether to hide compacted notes
    pub fn with_hide_compacted(mut self, hide: bool) -> Self {
        self.hide_compacted = hide;
        self
    }

    /// Check if a note matches all configured filters
    pub fn matches(&self, note: &crate::note::Note, compaction_ctx: &CompactionContext) -> bool {
        if !self.matches_compaction(note, compaction_ctx) {
            return false;
        }

        if !self.matches_tag(note) {
            return false;
        }

        if !self.matches_type(note) {
            return false;
        }

        if !self.matches_since(note) {
            return false;
        }

        if !self.matches_min_value(note) {
            return false;
        }

        if !self.matches_custom(note) {
            return false;
        }

        true
    }

    /// Check compaction visibility
    fn matches_compaction(
        &self,
        note: &crate::note::Note,
        compaction_ctx: &CompactionContext,
    ) -> bool {
        if !self.hide_compacted {
            return true;
        }

        !compaction_ctx.is_compacted(&note.frontmatter.id)
    }

    /// Check tag filter
    fn matches_tag(&self, note: &crate::note::Note) -> bool {
        // If equivalent tags are set (for alias resolution), check against those
        if let Some(ref equiv_tags) = self.equivalent_tags {
            equiv_tags.iter().any(|t| note.frontmatter.tags.contains(t))
        } else if let Some(tag) = self.tag {
            // Otherwise, check against the single tag
            note.frontmatter.tags.iter().any(|t| t == tag)
        } else {
            true
        }
    }

    /// Check type filter
    fn matches_type(&self, note: &crate::note::Note) -> bool {
        if let Some(ref nt) = self.note_type {
            note.note_type().as_str() == nt.as_str()
        } else {
            true
        }
    }

    /// Check since filter
    fn matches_since(&self, note: &crate::note::Note) -> bool {
        if let Some(since) = self.since {
            note.frontmatter
                .created
                .is_some_and(|created| created >= since)
        } else {
            true
        }
    }

    /// Check min-value filter
    fn matches_min_value(&self, note: &crate::note::Note) -> bool {
        if let Some(min_val) = self.min_value {
            let value = note.frontmatter.value.unwrap_or(50);
            value >= min_val
        } else {
            true
        }
    }

    /// Check custom metadata filter
    ///
    /// Supports:
    /// - Equality: `key=value`
    /// - Existence: `key` (present), `!key` (absent)
    /// - Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
    fn matches_custom(&self, note: &crate::note::Note) -> bool {
        if let Some(custom_filter) = self.custom {
            matches_custom_filter(&note.frontmatter.custom, custom_filter)
        } else {
            true
        }
    }
}
