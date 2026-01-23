//! Status message formatting helpers

use crate::lib::error::Result;
use crate::lib::records::escape_quotes;
use serde_json::json;

/// Print a JSON status message with optional fields
///
/// # Examples
/// ```ignore
/// print_json_status("installed", Some("AGENTS.md created successfully"), &[("path", "AGENTS.md")])?;
/// ```
pub fn print_json_status(
    status: &str,
    message: Option<&str>,
    extra_fields: &[(&str, serde_json::Value)],
) -> Result<()> {
    let mut output = json!({ "status": status });

    if let Some(msg) = message {
        if let Some(obj) = output.as_object_mut() {
            obj.insert("message".to_string(), json!(msg));
        }
    }

    for (key, value) in extra_fields {
        if let Some(obj) = output.as_object_mut() {
            obj.insert(key.to_string(), value.clone());
        }
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Build compaction annotations string for Records format
///
/// Returns a string like " via=qp-123 compacts=5 compaction=80%"
pub fn build_compaction_annotations(
    via: Option<&str>,
    compacts_count: usize,
    compaction_pct: Option<f32>,
) -> String {
    let mut annotations = String::new();
    if let Some(via_id) = via {
        annotations.push_str(&format!(" via={}", via_id));
    }
    if compacts_count > 0 {
        annotations.push_str(&format!(" compacts={}", compacts_count));

        if let Some(pct) = compaction_pct {
            annotations.push_str(&format!(" compaction={:.0}%", pct));
        }
    }
    annotations
}

/// Format custom metadata value for display
///
/// Handles different YAML value types (string, number, bool, etc.)
pub fn format_custom_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => format!("\"{}\"", crate::lib::records::escape_quotes(s)),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        _ => format!("{:?}", value),
    }
}

/// Add compaction annotations to a JSON object
///
/// Adds compacts_count, compaction_pct, and optionally compacted_ids
pub fn add_compaction_to_json(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    compacts_count: usize,
    compaction_pct: Option<f32>,
    compacted_ids: Option<Vec<String>>,
    compacted_ids_truncated: bool,
) {
    if compacts_count > 0 {
        obj.insert("compacts".to_string(), json!(compacts_count));

        if let Some(pct) = compaction_pct {
            obj.insert("compaction_pct".to_string(), json!(format!("{:.1}", pct)));
        }

        if let Some(ids) = compacted_ids {
            obj.insert("compacted_ids".to_string(), json!(ids));
            if compacted_ids_truncated {
                obj.insert("compacted_ids_truncated".to_string(), json!(true));
            }
        }
    }
}

/// Print a Records format header
///
/// # Examples
/// ```ignore
/// print_records_header("setup.install", &[("integration", "agents-md"), ("status", "installed")]);
/// ```
pub fn print_records_header(mode: &str, extra_fields: &[(&str, &str)]) {
    let mut parts = vec!["H qipu=1 records=1".to_string(), format!("mode={}", mode)];

    for (key, value) in extra_fields {
        parts.push(format!("{}={}", key, value));
    }

    println!("{}", parts.join(" "));
}

/// Wrap body content in Records format with B and B-END markers
///
/// # Examples
/// ```ignore
/// wrap_records_body(note_id, "Body content here");
/// ```
pub fn wrap_records_body(id: &str, body: &str) {
    println!("B {}", id);
    for line in body.lines() {
        println!("{}", line);
    }
    println!("B-END");
}

/// Print a Records format data line
///
/// # Examples
/// ```ignore
/// print_records_data("path", "AGENTS.md");
/// print_records_data("message", "File created successfully");
/// ```
pub fn print_records_data(key: &str, value: &str) {
    println!("D {} {}", key, escape_quotes(value));
}

/// Format tags as CSV string, using "-" for empty tags
///
/// # Examples
/// ```ignore
/// format_tags_csv(&["tag1", "tag2"]) // "tag1,tag2"
/// format_tags_csv(&[]) // "-"
/// ```
pub fn format_tags_csv(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(",")
    }
}

/// Format value as string, using "-" for None
///
/// # Examples
/// ```ignore
/// format_value(Some(42u8)) // "42"
/// format_value(None) // "-"
/// ```
pub fn format_value(value: Option<u8>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// Print a Records format summary line
///
/// Only outputs first line of summary per spec
///
/// # Examples
/// ```ignore
/// print_records_summary("qp-123", "This is a summary\nwith multiple lines");
/// ```
pub fn print_records_summary(id: &str, summary: &str) {
    if !summary.is_empty() {
        let first_line = summary.lines().next().unwrap_or(summary);
        println!("S {} {}", id, first_line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_json_status_basic() {
        let result = print_json_status("installed", Some("File created"), &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_status_with_fields() {
        let result = print_json_status(
            "installed",
            Some("File created"),
            &[("path", serde_json::json!("AGENTS.md"))],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_records_header_basic() {
        print_records_header("setup.install", &[]);
    }

    #[test]
    fn test_print_records_header_with_fields() {
        print_records_header("setup.install", &[("integration", "agents-md")]);
    }

    #[test]
    fn test_wrap_records_body() {
        wrap_records_body("qp-123", "Body line 1\nBody line 2");
    }
}
