use crate::compaction::CompactionContext;
use std::path::Path;

/// Utilities for records output format
/// Escape double quotes in a string for records format.
/// Replaces `"` with `\"` to allow safe embedding in quoted fields.
pub fn escape_quotes(s: &str) -> String {
    s.replace('\"', r#"\""#)
}

/// Convert an absolute path to a path relative to the current working directory
pub fn path_relative_to_cwd(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd)
            .ok()
            .map(|p| {
                let s = p.display().to_string();
                if s.is_empty() {
                    ".".to_string()
                } else {
                    s
                }
            })
            .unwrap_or_else(|| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

/// Options for compacted ID output
#[derive(Debug, Clone, Copy)]
pub struct CompactedIdOptions {
    /// Maximum depth to traverse for compacted IDs
    pub depth: u32,
    /// Maximum number of compacted IDs to include
    pub max_nodes: Option<usize>,
}

/// Output compacted IDs in records format
///
/// Returns formatted D-lines for compacted notes. If truncated, includes a D-line indicating truncation.
pub fn format_compacted_ids(
    note_id: &str,
    compaction_ctx: &CompactionContext,
    opts: &CompactedIdOptions,
) -> Vec<String> {
    let mut lines = Vec::new();
    let compacts_count = compaction_ctx.get_compacts_count(note_id);

    if compacts_count == 0 {
        return lines;
    }

    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(note_id, opts.depth, opts.max_nodes)
    {
        for id in &ids {
            lines.push(format!("D compacted {} from={}", id, note_id));
        }

        if truncated {
            let max = opts.max_nodes.unwrap_or(ids.len());
            lines.push(format!(
                "D compacted_truncated max={} total={}",
                max, compacts_count
            ));
        }
    }

    lines
}

/// Format a note record header line in records format
///
/// Returns a formatted N-line with note metadata.
pub fn format_note_record(
    note_id: &str,
    note_type: &str,
    title: &str,
    tags_csv: &str,
    annotations: &str,
) -> String {
    format!(
        "N {} {} \"{}\" tags={}{}",
        note_id,
        note_type,
        escape_quotes(title),
        tags_csv,
        annotations
    )
}

/// Format a summary line in records format
///
/// Returns a formatted S-line for a note summary.
pub fn format_summary_line(note_id: &str, summary: &str) -> String {
    format!("S {} {}", note_id, summary)
}

/// Format body lines in records format
///
/// Returns formatted B-lines with body content and B-END marker.
pub fn format_body_lines(note_id: &str, body: &str) -> Vec<String> {
    let mut lines = vec![format!("B {}", note_id)];
    lines.push(body.to_string());
    if !body.ends_with('\n') {
        lines.push("\n".to_string());
    }
    lines.push("B-END\n".to_string());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use std::path::PathBuf;

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_quotes("no quotes"), "no quotes");
        assert_eq!(escape_quotes(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(
            escape_quotes(r#"multiple "quotes" in "text""#),
            r#"multiple \"quotes\" in \"text\""#
        );
        assert_eq!(escape_quotes(""), "");
        assert_eq!(escape_quotes(r#""""#), r#"\"\""#);
    }

    #[test]
    fn test_path_relative_to_cwd() {
        let Ok(cwd) = std::env::current_dir() else {
            // Skip test if current directory is not available (test isolation issue)
            return;
        };

        // Test path that's exactly the CWD
        assert_eq!(path_relative_to_cwd(&cwd), ".");

        // Test path that's a subdirectory of CWD
        let subdir = cwd.join("subdir");
        assert_eq!(path_relative_to_cwd(&subdir), "subdir");

        // Test path that's a nested subdirectory
        let nested = cwd.join("a").join("b").join("c");
        assert_eq!(path_relative_to_cwd(&nested), "a/b/c");

        // Test absolute path outside CWD (should return absolute path as fallback)
        let other = if cfg!(unix) {
            PathBuf::from("/some/other/path")
        } else {
            PathBuf::from("C:\\some\\other\\path")
        };
        let result = path_relative_to_cwd(&other);
        assert!(
            result.starts_with("/") || result.contains(":"),
            "Path outside CWD should return absolute path"
        );
    }

    #[test]
    fn test_format_note_record() {
        let result = format_note_record("qp-abc123", "permanent", "Test Note", "tag1,tag2", "");
        assert_eq!(
            result,
            r#"N qp-abc123 permanent "Test Note" tags=tag1,tag2"#
        );
    }

    #[test]
    fn test_format_note_record_with_quotes_in_title() {
        let result = format_note_record(
            "qp-abc123",
            "permanent",
            r#"Note with "quotes""#,
            "tag1",
            "",
        );
        assert_eq!(
            result,
            r#"N qp-abc123 permanent "Note with \"quotes\"" tags=tag1"#
        );
    }

    #[test]
    fn test_format_note_record_with_annotations() {
        let result = format_note_record("qp-abc123", "permanent", "Test Note", "-", " compacts=2");
        assert_eq!(
            result,
            r#"N qp-abc123 permanent "Test Note" tags=- compacts=2"#
        );
    }

    #[test]
    fn test_format_summary_line() {
        let result = format_summary_line("qp-abc123", "Summary text");
        assert_eq!(result, "S qp-abc123 Summary text");
    }

    #[test]
    fn test_format_body_lines() {
        let body = "Line 1\nLine 2\n";
        let lines = format_body_lines("qp-abc123", body);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "B qp-abc123");
        assert_eq!(lines[1], body);
        assert_eq!(lines[2], "B-END\n");
    }

    #[test]
    fn test_format_body_lines_without_trailing_newline() {
        let body = "Line 1\nLine 2";
        let lines = format_body_lines("qp-abc123", body);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "B qp-abc123");
        assert_eq!(lines[1], body);
        assert_eq!(lines[2], "\n");
        assert_eq!(lines[3], "B-END\n");
    }

    #[test]
    fn test_format_compacted_ids_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), crate::store::InitOptions::default()).unwrap();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let opts = CompactedIdOptions {
            depth: 1,
            max_nodes: None,
        };

        let lines = format_compacted_ids(note.id(), &compaction_ctx, &opts);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_format_compacted_ids_with_compacts() {
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
        let opts = CompactedIdOptions {
            depth: 1,
            max_nodes: None,
        };

        let lines = format_compacted_ids(digest.id(), &compaction_ctx, &opts);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("D compacted "));
        assert!(lines[0].contains(&format!(" from={}", digest.id())));
    }

    #[test]
    fn test_format_compacted_ids_truncated() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), crate::store::InitOptions::default()).unwrap();

        let mut original_notes = Vec::new();
        for i in 0..5 {
            let note = store
                .create_note(&format!("Note {}", i), None, &[format!("tag{}", i)], None)
                .unwrap();
            original_notes.push(note);
        }

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = original_notes.iter().map(|n| n.id().to_string()).collect();
        store.save_note(&mut digest).unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let opts = CompactedIdOptions {
            depth: 1,
            max_nodes: Some(2),
        };

        let lines = format_compacted_ids(digest.id(), &compaction_ctx, &opts);
        assert!(lines.len() > 2);
        assert!(lines[2].contains("compacted_truncated"));
        assert!(lines[2].contains("max=2"));
    }
}
