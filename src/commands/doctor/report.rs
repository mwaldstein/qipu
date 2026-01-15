use super::types::{DoctorResult, Severity};
use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::store::Store;

/// Output the doctor result in the appropriate format
pub fn output_result(cli: &Cli, store: &Store, result: &DoctorResult) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Human => {
            if result.issues.is_empty() {
                if !cli.quiet {
                    println!("Store is healthy ({} notes scanned)", result.notes_scanned);
                }
            } else {
                println!(
                    "Found {} issue(s) in {} notes:",
                    result.issues.len(),
                    result.notes_scanned
                );
                println!();

                for issue in &result.issues {
                    let severity_prefix = match issue.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARN ",
                    };

                    let fixable_suffix = if issue.fixable { " [fixable]" } else { "" };

                    println!(
                        "  {} [{}] {}{}",
                        severity_prefix, issue.category, issue.message, fixable_suffix
                    );

                    if let Some(path) = &issue.path {
                        if issue.note_id.is_none() {
                            println!("         at {}", path);
                        }
                    }
                }

                println!();
                println!(
                    "Summary: {} error(s), {} warning(s)",
                    result.error_count, result.warning_count
                );

                if result.fixed_count > 0 {
                    println!("Fixed {} issue(s)", result.fixed_count);
                }
            }
        }
        OutputFormat::Records => {
            // Header
            println!(
                "H qipu=1 records=1 store={} mode=doctor notes={} errors={} warnings={}",
                store_path_for_records(store),
                result.notes_scanned,
                result.error_count,
                result.warning_count
            );

            // Issues as diagnostic lines (D prefix)
            for issue in &result.issues {
                let note_part = issue
                    .note_id
                    .as_ref()
                    .map(|id| format!(" note={}", id))
                    .unwrap_or_default();

                println!(
                    "D {} {} \"{}\"{}",
                    issue.severity, issue.category, issue.message, note_part
                );
            }
        }
    }

    Ok(())
}

/// Get store path for records output (helper to work around borrow issues)
fn store_path_for_records(store: &Store) -> String {
    store.root().display().to_string()
}
