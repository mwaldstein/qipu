use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::store::Store;

/// Execute `qipu compact apply`
pub fn execute(
    cli: &Cli,
    root: &Path,
    digest_id: &str,
    note_ids: &[String],
    from_stdin: bool,
    notes_file: Option<&PathBuf>,
) -> Result<()> {
    let start = Instant::now();
    // Discover store
    if cli.verbose {
        debug!(
            digest_id,
            note_count = note_ids.len(),
            from_stdin,
            notes_file_provided = notes_file.is_some(),
            "apply_params"
        );
    }

    let store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(root)?
    };

    if cli.verbose {
        debug!(store = %store.root().display(), "discover_store");
    }

    // Collect source note IDs from various sources
    let mut source_ids = note_ids.to_vec();

    // Read from stdin if requested
    if from_stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                source_ids.push(trimmed.to_string());
            }
        }
    }

    // Read from file if requested
    if let Some(file_path) = notes_file {
        let content = fs::read_to_string(file_path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                source_ids.push(trimmed.to_string());
            }
        }
    }

    if source_ids.is_empty() {
        return Err(qipu_core::error::QipuError::UsageError(
            "no source note IDs provided (use --note, --from-stdin, or --notes-file)".to_string(),
        ));
    }

    // Load the digest note
    let mut digest_note = store.get_note(digest_id)?;

    // Sort source IDs for deterministic ordering
    source_ids.sort();
    source_ids.dedup();

    // Update compacts field (idempotent - merge with existing)
    let mut existing_compacts: std::collections::HashSet<String> =
        digest_note.frontmatter.compacts.iter().cloned().collect();
    for id in &source_ids {
        existing_compacts.insert(id.clone());
    }
    let mut new_compacts: Vec<String> = existing_compacts.into_iter().collect();
    new_compacts.sort();
    digest_note.frontmatter.compacts = new_compacts;

    // Validate invariants before saving (include updated digest compacts)
    let mut all_notes = store.list_notes()?;
    let mut replaced = false;
    for note in &mut all_notes {
        if note.frontmatter.id == digest_note.frontmatter.id {
            *note = digest_note.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        all_notes.push(digest_note.clone());
    }

    let ctx = CompactionContext::build(&all_notes)?;
    let errors = ctx.validate(&all_notes);
    if !errors.is_empty() {
        tracing::warn!("Compaction validation errors:");
        for err in &errors {
            tracing::warn!("  - {}", err);
        }
        return Err(qipu_core::error::QipuError::Other(
            "compaction invariants violated".to_string(),
        ));
    }

    // Save the digest note
    store.save_note(&mut digest_note)?;

    if cli.verbose {
        debug!(
            digest_id,
            compact_count = source_ids.len(),
            elapsed = ?start.elapsed(),
            "apply_compaction"
        );
    }

    // Output
    match cli.format {
        qipu_core::format::OutputFormat::Human => {
            println!("Applied compaction:");
            println!("  Digest: {}", digest_id);
            println!("  Compacts {} notes:", source_ids.len());
            for id in &source_ids {
                println!("    - {}", id);
            }
        }
        qipu_core::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "digest_id": digest_id,
                "compacts": source_ids,
                "count": source_ids.len(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        qipu_core::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.apply digest={} count={}",
                digest_id,
                source_ids.len()
            );
            for id in &source_ids {
                println!("D compacted {}", id);
            }
        }
    }

    Ok(())
}
