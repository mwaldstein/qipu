//! Note filtering utilities

use crate::lib::compaction::CompactionContext;
use crate::lib::note::NoteType;
use chrono::{DateTime, Utc};
use serde_yaml::Value;

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
    pub fn matches(
        &self,
        note: &crate::lib::note::Note,
        compaction_ctx: &CompactionContext,
    ) -> bool {
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
        note: &crate::lib::note::Note,
        compaction_ctx: &CompactionContext,
    ) -> bool {
        if !self.hide_compacted {
            return true;
        }

        !compaction_ctx.is_compacted(&note.frontmatter.id)
    }

    /// Check tag filter
    fn matches_tag(&self, note: &crate::lib::note::Note) -> bool {
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
    fn matches_type(&self, note: &crate::lib::note::Note) -> bool {
        if let Some(ref nt) = self.note_type {
            note.note_type().as_str() == nt.as_str()
        } else {
            true
        }
    }

    /// Check since filter
    fn matches_since(&self, note: &crate::lib::note::Note) -> bool {
        if let Some(since) = self.since {
            note.frontmatter
                .created
                .is_some_and(|created| created >= since)
        } else {
            true
        }
    }

    /// Check min-value filter
    fn matches_min_value(&self, note: &crate::lib::note::Note) -> bool {
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
    fn matches_custom(&self, note: &crate::lib::note::Note) -> bool {
        if let Some(custom_filter) = self.custom {
            let expr = custom_filter.trim();

            // Check for absence (!key)
            if let Some(key) = expr.strip_prefix('!') {
                let key = key.trim();
                return !key.is_empty() && !note.frontmatter.custom.contains_key(key);
            }

            // Check for numeric comparisons (key>n, key>=n, key<n, key<=n) - must be checked before equality!
            if let Some((k, v)) = expr.split_once(">=") {
                let key = k.trim();
                let value = v.trim();
                !key.is_empty()
                    && !value.is_empty()
                    && self.match_numeric_comparison(note, key, value, |a, b| a >= b)
            } else if let Some((k, v)) = expr.split_once('>') {
                let key = k.trim();
                let value = v.trim();
                !key.is_empty()
                    && !value.is_empty()
                    && self.match_numeric_comparison(note, key, value, |a, b| a > b)
            } else if let Some((k, v)) = expr.split_once("<=") {
                let key = k.trim();
                let value = v.trim();
                !key.is_empty()
                    && !value.is_empty()
                    && self.match_numeric_comparison(note, key, value, |a, b| a <= b)
            } else if let Some((k, v)) = expr.split_once('<') {
                let key = k.trim();
                let value = v.trim();
                !key.is_empty()
                    && !value.is_empty()
                    && self.match_numeric_comparison(note, key, value, |a, b| a < b)
            } else if let Some((key, value)) = expr.split_once('=') {
                // Equality check (key=value)
                let key = key.trim();
                let value = value.trim();
                !key.is_empty()
                    && note
                        .frontmatter
                        .custom
                        .get(key)
                        .map(|v| self.match_custom_value(v, value))
                        .unwrap_or(false)
            } else {
                // No comparison operator found, check for existence
                let key = expr.trim();
                !key.is_empty() && note.frontmatter.custom.contains_key(key)
            }
        } else {
            true
        }
    }

    /// Match a custom value against the filter value
    fn match_custom_value(&self, yaml_value: &Value, filter_value: &str) -> bool {
        match yaml_value {
            Value::String(s) => s == filter_value,
            Value::Number(num) => num.to_string() == filter_value,
            Value::Bool(b) => b.to_string() == filter_value,
            _ => false,
        }
    }

    /// Match a numeric comparison against a custom field
    fn match_numeric_comparison<F>(
        &self,
        note: &crate::lib::note::Note,
        key: &str,
        value: &str,
        compare_fn: F,
    ) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let target_value: f64 = match value.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        note.frontmatter
            .custom
            .get(key)
            .and_then(|v| match v {
                Value::Number(num) => num.as_f64(),
                Value::String(s) => s.parse::<f64>().ok(),
                Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            })
            .map(|actual_value| compare_fn(actual_value, target_value))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::{Note, NoteFrontmatter};
    use chrono::Duration;
    use std::path::PathBuf;

    fn create_test_note(
        id: &str,
        title: &str,
        tags: Vec<String>,
        note_type: Option<NoteType>,
        created: Option<DateTime<Utc>>,
        value: Option<u8>,
    ) -> Note {
        Note {
            frontmatter: NoteFrontmatter {
                id: id.to_string(),
                title: title.to_string(),
                tags,
                created,
                updated: None,
                note_type,
                compacts: vec![],
                sources: vec![],
                links: vec![],
                summary: None,
                source: None,
                author: None,
                generated_by: None,
                prompt_hash: None,
                verified: None,
                value,
                custom: std::collections::HashMap::new(),
            },
            body: String::new(),
            path: Some(PathBuf::from(format!("{}.md", id))),
        }
    }

    #[test]
    fn test_filter_with_tag() {
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec!["matching".to_string()],
            None,
            None,
            None,
        );

        let filter = NoteFilter::new().with_tag(Some("matching"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_without_tag() {
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec!["other".to_string()],
            None,
            None,
            None,
        );

        let filter = NoteFilter::new().with_tag(Some("matching"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_type() {
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec![],
            Some(NoteType::from(NoteType::PERMANENT)),
            None,
            None,
        );

        let filter = NoteFilter::new().with_type(Some(NoteType::from(NoteType::PERMANENT)));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_without_type() {
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec![],
            Some(NoteType::from(NoteType::FLEETING)),
            None,
            None,
        );

        let filter = NoteFilter::new().with_type(Some(NoteType::from(NoteType::PERMANENT)));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_since() {
        let now = Utc::now();
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec![],
            None,
            Some(now - Duration::days(1)),
            None,
        );

        let filter = NoteFilter::new().with_since(Some(now - Duration::days(5)));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_without_since() {
        let now = Utc::now();
        let note = create_test_note(
            "qp-abc",
            "Test Note",
            vec![],
            None,
            Some(now - Duration::days(10)),
            None,
        );

        let filter = NoteFilter::new().with_since(Some(now - Duration::days(5)));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_min_value() {
        let note = create_test_note("qp-abc", "Test Note", vec![], None, None, Some(75));

        let filter = NoteFilter::new().with_min_value(Some(50));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_without_min_value() {
        let note = create_test_note("qp-abc", "Test Note", vec![], None, None, Some(30));

        let filter = NoteFilter::new().with_min_value(Some(50));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_min_value_default() {
        let note = create_test_note("qp-abc", "Test Note", vec![], None, None, None);

        let filter = NoteFilter::new().with_min_value(Some(50));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    fn create_note_with_custom(custom: std::collections::HashMap<String, Value>) -> Note {
        Note {
            frontmatter: NoteFrontmatter {
                id: "qp-abc".to_string(),
                title: "Test Note".to_string(),
                tags: vec![],
                created: None,
                updated: None,
                note_type: None,
                compacts: vec![],
                sources: vec![],
                links: vec![],
                summary: None,
                source: None,
                author: None,
                generated_by: None,
                prompt_hash: None,
                verified: None,
                value: None,
                custom,
            },
            body: String::new(),
            path: Some(PathBuf::from("qp-abc.md")),
        }
    }

    #[test]
    fn test_filter_with_custom_string() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("key".to_string(), Value::String("value".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("key=value"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_number() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "count".to_string(),
            Value::Number(serde_yaml::Number::from(42)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("count=42"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_bool() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("flag".to_string(), Value::Bool(true));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("flag=true"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_mismatch() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("key".to_string(), Value::String("other".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("key=value"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_exists() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("key".to_string(), Value::String("value".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("key"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_not_exists() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("key".to_string(), Value::String("value".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("!other"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_absent() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("key".to_string(), Value::String("value".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("!key"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_greater_than() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "priority".to_string(),
            Value::Number(serde_yaml::Number::from(10)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority>5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_less_than() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "priority".to_string(),
            Value::Number(serde_yaml::Number::from(3)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority<5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_greater_equal() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "priority".to_string(),
            Value::Number(serde_yaml::Number::from(5)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority>=5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_less_equal() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "priority".to_string(),
            Value::Number(serde_yaml::Number::from(5)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority<=5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_fails() {
        let mut custom = std::collections::HashMap::new();
        custom.insert(
            "priority".to_string(),
            Value::Number(serde_yaml::Number::from(3)),
        );
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority>5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(!filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_string_value() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("priority".to_string(), Value::String("10".to_string()));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("priority>5"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_bool_true() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("active".to_string(), Value::Bool(true));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("active>0"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }

    #[test]
    fn test_filter_with_custom_numeric_bool_false() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("active".to_string(), Value::Bool(false));
        let note = create_note_with_custom(custom);

        let filter = NoteFilter::new().with_custom(Some("active<1"));
        let compaction_ctx = CompactionContext::build(&[]).unwrap();

        assert!(filter.matches(&note, &compaction_ctx));
    }
}
