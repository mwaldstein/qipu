//! Output format handling for qipu
//!
//! Supports three output formats per spec (specs/cli-tool.md):
//! - human: Readable, concise output for terminal use
//! - json: Stable, machine-readable JSON
//! - records: Line-oriented format optimized for LLM context injection

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;

use crate::compaction::CompactionContext;
use crate::error::QipuError;
use crate::note::Note;

/// Compaction output formatting options
#[derive(Debug, Clone, Copy, Default)]
pub struct CompactionOutputOptions {
    /// Include compacted note IDs in output
    pub with_compaction_ids: bool,
    /// Compaction traversal depth (requires with_compaction_ids)
    pub compaction_depth: Option<u32>,
    /// Maximum compacted notes to include in output
    pub compaction_max_nodes: Option<usize>,
}

/// Output format for qipu commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable output (default)
    #[default]
    Human,
    /// JSON output for machine consumption
    Json,
    /// Records output for LLM context injection
    Records,
}

impl FromStr for OutputFormat {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "records" => Ok(OutputFormat::Records),
            other => Err(QipuError::UnknownFormat(other.to_string())),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Human => write!(f, "human"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Records => write!(f, "records"),
        }
    }
}

/// Build compaction annotations string for a note
///
/// Returns a formatted string containing compaction count and percentage
/// if the note has compactions.
pub fn build_compaction_annotations(
    note_id: &str,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) -> String {
    let mut annotations = String::new();
    let compacts_count = compaction_ctx.get_compacts_count(note_id);

    if compacts_count > 0 {
        annotations.push_str(&format!(" compacts={}", compacts_count));

        if let Some(note) = note_map.get(note_id) {
            if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
                annotations.push_str(&format!(" compaction={:.0}%", pct));
            }
        }
    }

    annotations
}

/// Output compacted IDs in human-readable format
///
/// Prints a list of compacted note IDs with truncation indicator if needed.
pub fn output_compaction_ids(
    opts: &CompactionOutputOptions,
    note_id: &str,
    compaction_ctx: &CompactionContext,
) {
    if !opts.with_compaction_ids {
        return;
    }

    let compacts_count = compaction_ctx.get_compacts_count(note_id);
    if compacts_count == 0 {
        return;
    }

    let depth = opts.compaction_depth.unwrap_or(1);
    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(note_id, depth, opts.compaction_max_nodes)
    {
        let ids_str = ids.join(", ");
        let suffix = if truncated {
            let max = opts.compaction_max_nodes.unwrap_or(ids.len());
            format!(" (truncated, showing {} of {})", max, compacts_count)
        } else {
            String::new()
        };
        println!("  Compacted: {}{}", ids_str, suffix);
    }
}

/// Add compaction annotations to a JSON object
///
/// Adds compacts count, compaction percentage, and optionally compacted IDs
/// to the JSON object.
pub fn add_compaction_to_json(
    opts: &CompactionOutputOptions,
    note_id: &str,
    json: &mut Value,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) {
    let compacts_count = compaction_ctx.get_compacts_count(note_id);
    if compacts_count == 0 {
        return;
    }

    if let Some(obj) = json.as_object_mut() {
        obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

        if let Some(note) = note_map.get(note_id) {
            if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
                obj.insert(
                    "compaction_pct".to_string(),
                    serde_json::json!(format!("{:.1}", pct)),
                );
            }
        }

        add_compacted_ids_to_json(opts, note_id, obj, compaction_ctx);
    }
}

/// Add compacted IDs to a JSON object
///
/// Adds the list of compacted note IDs and truncation indicator
/// to the JSON object if requested.
pub fn add_compacted_ids_to_json(
    opts: &CompactionOutputOptions,
    note_id: &str,
    obj: &mut Map<String, Value>,
    compaction_ctx: &CompactionContext,
) {
    if !opts.with_compaction_ids {
        return;
    }

    let depth = opts.compaction_depth.unwrap_or(1);
    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(note_id, depth, opts.compaction_max_nodes)
    {
        obj.insert("compacted_ids".to_string(), serde_json::json!(ids));

        if truncated {
            obj.insert(
                "compacted_ids_truncated".to_string(),
                serde_json::json!(true),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            "human".parse::<OutputFormat>().unwrap(),
            OutputFormat::Human
        );
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "records".parse::<OutputFormat>().unwrap(),
            OutputFormat::Records
        );
        assert_eq!(
            "HUMAN".parse::<OutputFormat>().unwrap(),
            OutputFormat::Human
        );
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    }

    #[test]
    fn test_unknown_format() {
        let err = "invalid".parse::<OutputFormat>().unwrap_err();
        assert!(matches!(err, QipuError::UnknownFormat(_)));
    }

    #[test]
    fn test_format_display() {
        assert_eq!(OutputFormat::Human.to_string(), "human");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Records.to_string(), "records");
    }

    #[test]
    fn test_build_compaction_annotations_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), crate::store::InitOptions::default()).unwrap();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let annotations = build_compaction_annotations(note.id(), &compaction_ctx, &note_map);
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_build_compaction_annotations_with_compacts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), crate::store::InitOptions::default()).unwrap();

        let note1 = store
            .create_note_with_content(
                "Original Note",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note.",
                None,
            )
            .unwrap();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1.id().to_string()];
        store.save_note(&mut digest).unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let digest_note = all_notes
            .iter()
            .find(|n| n.frontmatter.compacts.iter().any(|id| id == note1.id()))
            .unwrap();
        let annotations =
            build_compaction_annotations(digest_note.id(), &compaction_ctx, &note_map);

        assert!(annotations.contains("compacts=1"));
        assert!(annotations.contains("compaction="));
    }
}
