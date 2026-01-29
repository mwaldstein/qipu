use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::store::Store;

/// Execute `qipu compact status`
pub fn execute(cli: &Cli, note_id: &str) -> Result<()> {
    let start = Instant::now();
    if cli.verbose {
        debug!(note_id, "status_params");
    }

    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(&root)?
    };

    if cli.verbose {
        debug!(store = %store.root().display(), "discover_store");
    }

    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;

    if cli.verbose {
        debug!(note_count = all_notes.len(), "build_compaction_context");
    }

    // Get compaction status
    let canonical = ctx.canon(note_id)?;
    let direct_compactor = ctx.get_compactor(note_id);
    let compacted_notes = ctx.get_compacted_notes(note_id);

    if cli.verbose {
        debug!(
            note_id,
            ?canonical,
            ?direct_compactor,
            compacted_count = compacted_notes.as_ref().map(|c| c.len()).unwrap_or(0),
            elapsed = ?start.elapsed(),
            "compaction_status"
        );
    }

    // Output
    match cli.format {
        qipu_core::format::OutputFormat::Human => {
            let note = store.get_note(note_id)?;
            println!("Note: {} ({})", note.frontmatter.title, note_id);
            println!();

            if let Some(compactor) = direct_compactor {
                let compactor_note = store.get_note(compactor)?;
                println!(
                    "  Compacted by: {} ({})",
                    compactor_note.frontmatter.title, compactor
                );
            } else {
                println!("  Compacted by: (none)");
            }

            if canonical != note_id {
                let canonical_note = store.get_note(&canonical)?;
                println!(
                    "  Canonical: {} ({})",
                    canonical_note.frontmatter.title, canonical
                );
            } else {
                println!("  Canonical: (self)");
            }

            if let Some(compacts) = compacted_notes {
                println!("  Compacts {} notes:", compacts.len());
                for id in compacts {
                    if let Ok(n) = store.get_note(id) {
                        println!("    - {} ({})", n.frontmatter.title, id);
                    }
                }
            } else {
                println!("  Compacts: (none)");
            }
        }
        qipu_core::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "note_id": note_id,
                "compactor": direct_compactor,
                "canonical": canonical,
                "compacts": compacted_notes,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        qipu_core::format::OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=compact.status note={}", note_id);
            if let Some(compactor) = direct_compactor {
                println!("D compactor {}", compactor);
            }
            if canonical != note_id {
                println!("D canonical {}", canonical);
            }
            if let Some(compacts) = compacted_notes {
                for id in compacts {
                    println!("D compacts {}", id);
                }
            }
        }
    }

    Ok(())
}
