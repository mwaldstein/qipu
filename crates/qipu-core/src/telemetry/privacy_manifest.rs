//! Privacy manifest and documentation generation

use std::fmt::Write;

pub const PRIVACY_MANIFEST: &str = r#"
Qipu Telemetry Privacy Manifest
================================

This document describes what telemetry data Qipu collects, how it is used, 
and how you can control it.

WHAT IS COLLECTED
-----------------

We collect ONLY the following anonymized, aggregated data:

1. COMMAND EXECUTION
   - Command names (e.g., "capture", "list", "search")
   - Success/failure status (counts only, no individual outcomes)
   - Execution duration in buckets (<100ms, 100-500ms, 500ms-1s, 1-5s, 5s+)
   - Error types (generic categories like "IOError", "ParseError")

2. ENVIRONMENT STATE
   - Operating system platform (Linux, macOS, Windows) - generic only
   - Application version
   - Workspace count in tiers (0, 1, 2-5, 6-20, 20+)
   - Total note count in tiers (<10, 10-100, 100-1000, 1000-10000, 10000+)

WHAT IS NOT COLLECTED
---------------------

We NEVER collect:

- Personal information (names, email addresses, identifiers)
- Note content (text, markdown, attachments)
- File names or paths
- Search queries
- User-provided arguments or flags
- Unique session or user identifiers
- Timestamps at sub-daily granularity
- Sequential ordering that could reconstruct workflows
- Stack traces containing user paths or data

AGGREGATION & PRIVACY
---------------------

All data is aggregated BEFORE upload:

1. Individual events are grouped by day
2. Commands are counted per type (e.g., "list: 15 times today")
3. Durations are bucketed (no precise timing)
4. Statistics are summarized (counts only)

This means no single command execution can be identified from uploaded data.

DATA RETENTION
--------------

Local Storage:
- Raw events: 7 days maximum
- Aggregated sessions: Pending upload only
- After successful upload: deleted from local storage

Remote Storage:
- TBD (will be documented when endpoint is available)

HOW TO DISABLE TELEMETRY
--------------------------

Disable globally:
  export QIPU_NO_TELEMETRY=1

Disable via CLI (future):
  qipu telemetry disable

Disable via config (future):
  Set "telemetry_enabled = false" in your config file

VIEW WHAT WOULD BE SENT
-----------------------

Run the following command to see what data is pending upload:

  qipu telemetry show

This displays aggregated session data WITHOUT actually uploading it.

WHY WE COLLECT THIS DATA
-------------------------

The data helps us:
- Understand which features are most used
- Identify performance bottlenecks
- Improve error handling
- Prioritize future development

We strictly limit collection to what's necessary for these goals.

PRIVACY REVIEW
--------------

This telemetry system has been designed with privacy-first principles:
- No PII or content collection
- Aggregation before upload
- Easy opt-out mechanisms
- Transparent documentation
- Minimal data collection

For questions or concerns, please open an issue on GitHub.

Last updated: 2026-01-31
"#;

#[derive(Debug, Clone)]
pub struct PrivacyManifest {
    pub title: String,
    pub version: String,
    pub last_updated: String,
    pub collected_data: Vec<String>,
    pub not_collected_data: Vec<String>,
    pub retention_days: u32,
    pub aggregation_method: String,
    pub opt_out_methods: Vec<String>,
}

impl Default for PrivacyManifest {
    fn default() -> Self {
        Self {
            title: "Qipu Telemetry Privacy Manifest".to_string(),
            version: "1.0".to_string(),
            last_updated: "2026-01-31".to_string(),
            collected_data: vec![
                "Command names (e.g., capture, list, search)".to_string(),
                "Success/failure status (counts only)".to_string(),
                "Execution duration in buckets".to_string(),
                "Error type categories (IOError, ParseError, etc.)".to_string(),
                "OS platform (Linux, macOS, Windows)".to_string(),
                "Application version".to_string(),
                "Workspace count tiers (0, 1, 2-5, 6-20, 20+)".to_string(),
                "Note count tiers (<10, 10-100, 100-1000, 1000-10000, 10000+)".to_string(),
            ],
            not_collected_data: vec![
                "Personal information (names, email, identifiers)".to_string(),
                "Note content (text, markdown, attachments)".to_string(),
                "File names or paths".to_string(),
                "Search queries".to_string(),
                "User-provided arguments or flags".to_string(),
                "Unique session or user identifiers".to_string(),
                "Sub-daily timestamp granularity".to_string(),
                "Sequential ordering that could reconstruct workflows".to_string(),
                "Stack traces containing user paths or data".to_string(),
            ],
            retention_days: 7,
            aggregation_method: "Events grouped by day, counted by type, bucketed by duration"
                .to_string(),
            opt_out_methods: vec![
                "Environment variable: QIPU_NO_TELEMETRY=1".to_string(),
                "CLI command: qipu telemetry disable (future)".to_string(),
                "Config file: telemetry_enabled = false (future)".to_string(),
            ],
        }
    }
}

impl PrivacyManifest {
    pub fn display(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", self.title).unwrap();
        writeln!(output, "{}", "=".repeat(self.title.len())).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "Version: {}", self.version).unwrap();
        writeln!(output, "Last Updated: {}", self.last_updated).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "WHAT IS COLLECTED").unwrap();
        writeln!(output, "{}", "-".repeat(18)).unwrap();
        for item in &self.collected_data {
            writeln!(output, "  - {}", item).unwrap();
        }
        writeln!(output).unwrap();

        writeln!(output, "WHAT IS NOT COLLECTED").unwrap();
        writeln!(output, "{}", "-".repeat(21)).unwrap();
        for item in &self.not_collected_data {
            writeln!(output, "  - {}", item).unwrap();
        }
        writeln!(output).unwrap();

        writeln!(output, "DATA RETENTION").unwrap();
        writeln!(output, "{}", "-".repeat(15)).unwrap();
        writeln!(
            output,
            "  Local storage: {} days maximum",
            self.retention_days
        )
        .unwrap();
        writeln!(output, "  Remote storage: TBD").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "AGGREGATION METHOD").unwrap();
        writeln!(output, "{}", "-".repeat(20)).unwrap();
        writeln!(output, "  {}", self.aggregation_method).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "HOW TO DISABLE").unwrap();
        writeln!(output, "{}", "-".repeat(15)).unwrap();
        for method in &self.opt_out_methods {
            writeln!(output, "  - {}", method).unwrap();
        }
        writeln!(output).unwrap();

        output
    }

    pub fn get_raw_manifest() -> &'static str {
        PRIVACY_MANIFEST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_display() {
        let manifest = PrivacyManifest::default();
        let display = manifest.display();

        assert!(display.contains("WHAT IS COLLECTED"));
        assert!(display.contains("WHAT IS NOT COLLECTED"));
        assert!(display.contains("DATA RETENTION"));
        assert!(display.contains("HOW TO DISABLE"));
    }

    #[test]
    fn test_manifest_no_pii_in_collected() {
        let manifest = PrivacyManifest::default();

        for item in &manifest.collected_data {
            assert!(
                !item.contains("personal"),
                "Found PII in collected data: {}",
                item
            );
            assert!(
                !item.contains("email"),
                "Found email in collected data: {}",
                item
            );
            assert!(
                !item.contains("user "),
                "Found user identifier in collected data: {}",
                item
            );
        }
    }

    #[test]
    fn test_manifest_excludes_content() {
        let manifest = PrivacyManifest::default();

        for item in &manifest.collected_data {
            assert!(
                !item.contains("content"),
                "Found content in collected data: {}",
                item
            );
            assert!(
                !item.contains("text"),
                "Found text in collected data: {}",
                item
            );
            assert!(
                !item.contains("markdown"),
                "Found markdown in collected data: {}",
                item
            );
        }
    }

    #[test]
    fn test_manifest_excludes_identifiers() {
        let manifest = PrivacyManifest::default();

        for item in &manifest.collected_data {
            assert!(
                !item.contains("session") || item.contains("count"),
                "Found session identifier in collected data: {}",
                item
            );
            assert!(
                !item.contains("user"),
                "Found user identifier in collected data: {}",
                item
            );
        }
    }

    #[test]
    fn test_raw_manifest_exists() {
        let manifest = PrivacyManifest::get_raw_manifest();
        assert!(!manifest.is_empty());
        assert!(manifest.contains("WHAT IS COLLECTED"));
        assert!(manifest.contains("WHAT IS NOT COLLECTED"));
    }
}
