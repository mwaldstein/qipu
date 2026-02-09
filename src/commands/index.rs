//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate
//! //!
#![allow(clippy::if_same_then_else)]

mod filters;
mod formatters;
mod progress;

use crate::cli::Cli;
use crate::commands::format::dispatch_format;
use filters::{filter_by_moc, filter_by_recent, filter_quick_index, parse_modified_since};
use formatters::{IndexFormatter, IndexStatusFormatter};
use progress::ProgressTracker;
use qipu_core::error::{QipuError, Result};
use qipu_core::index::links;
use qipu_core::note::{Note, NoteType};
use qipu_core::store::Store;

/// Execute index command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    rebuild: bool,
    resume: bool,
    rewrite_wiki_links: bool,
    quick: bool,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    recent: Option<usize>,
    moc: Option<&str>,
    status: bool,
    basic: bool,
    full: bool,
    modified_since: Option<&str>,
    batch: Option<usize>,
) -> Result<()> {
    if status {
        return show_index_status(cli, store);
    }

    // Validate mutually exclusive flags
    if basic && full {
        return Err(QipuError::Other(
            "--basic and --full are mutually exclusive".to_string(),
        ));
    }

    let mut notes = store.list_notes()?;

    if rewrite_wiki_links {
        let mut rewritten_count = 0;
        for note in &mut notes {
            if links::rewrite_wiki_links(note)? {
                store.save_note(note)?;
                rewritten_count += 1;
            }
        }
        if !cli.quiet && rewritten_count > 0 {
            eprintln!("Rewrote wiki-links in {} notes", rewritten_count);
        }
    }

    let notes_count = notes.len();

    let interrupted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let interrupted_clone = std::sync::Arc::clone(&interrupted);
    let _ = ctrlc::set_handler(move || {
        interrupted_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    // Determine if we should use selective indexing
    let use_selective = quick
        || tag.is_some()
        || note_type.is_some()
        || recent.is_some()
        || moc.is_some()
        || modified_since.is_some();

    if cli.verbose {
        eprintln!("Indexing notes from .qipu/notes/...");
        let mut tracker = ProgressTracker::new();
        let mut progress = |indexed: usize, total: usize, note: &Note| {
            tracker.update(indexed, total, note);
        };

        let result = if use_selective {
            selective_index(
                cli,
                store,
                quick,
                tag,
                note_type,
                recent,
                moc,
                modified_since,
            )
        } else if resume {
            store
                .db()
                .rebuild_resume(store.root(), Some(&mut progress), Some(&interrupted), batch)
        } else if rebuild || full {
            store
                .db()
                .rebuild(store.root(), Some(&mut progress), Some(&interrupted), batch)
        } else if basic {
            store.db().rebuild_basic(store.root())
        } else {
            store
                .db()
                .incremental_repair(store.root(), Some(&mut progress), Some(&interrupted))
        };

        match result {
            Ok(_) => {
                // Success - continue to output formatting
            }
            Err(QipuError::Interrupted) => {
                eprintln!("Index interrupted. Run `qipu index --resume` to resume.");
                return Err(QipuError::Interrupted);
            }
            Err(e) => return Err(e),
        }
    } else {
        let result = if use_selective {
            selective_index(
                cli,
                store,
                quick,
                tag,
                note_type,
                recent,
                moc,
                modified_since,
            )
        } else if resume {
            store
                .db()
                .rebuild_resume(store.root(), None, Some(&interrupted), batch)
        } else if rebuild || full {
            store
                .db()
                .rebuild(store.root(), None, Some(&interrupted), batch)
        } else if basic {
            store.db().rebuild_basic(store.root())
        } else {
            store
                .db()
                .incremental_repair(store.root(), None, Some(&interrupted))
        };

        match result {
            Ok(_) => {
                // Success - continue to output formatting
            }
            Err(QipuError::Interrupted) => {
                eprintln!("Index interrupted. Run `qipu index --resume` to resume.");
                return Err(QipuError::Interrupted);
            }
            Err(e) => return Err(e),
        }
    }

    if !cli.quiet || cli.format != crate::cli::OutputFormat::Human {
        dispatch_format(cli, &IndexFormatter { store, notes_count })?;
    }

    Ok(())
}

fn show_index_status(cli: &Cli, store: &Store) -> Result<()> {
    let db_count = store.db().get_note_count().unwrap_or(0);
    let basic_count = store.db().count_basic_indexed().unwrap_or(0);
    let full_count = store.db().count_full_indexed().unwrap_or(0);

    dispatch_format(
        cli,
        &IndexStatusFormatter {
            store,
            db_count,
            basic_count,
            full_count,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn selective_index(
    cli: &Cli,
    store: &Store,
    quick: bool,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    recent: Option<usize>,
    moc: Option<&str>,
    modified_since: Option<&str>,
) -> Result<()> {
    use qipu_core::note::Note;

    let mut notes: Vec<Note> = store.list_notes()?;

    if quick {
        notes = filter_quick_index(store, &notes);
    }

    if let Some(moc_id) = moc {
        notes = filter_by_moc(store, &notes, moc_id);
    }

    if let Some(t) = tag {
        notes.retain(|n| n.frontmatter.tags.iter().any(|tag| tag == t));
    }

    if let Some(nt) = note_type {
        notes.retain(|n| n.note_type() == nt);
    }

    if let Some(n) = recent {
        notes = filter_by_recent(&notes, n);
    }

    if let Some(time_str) = modified_since {
        let cutoff = parse_modified_since(time_str)?;
        notes.retain(|n| {
            if let Some(path) = &n.path {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        return mtime >= cutoff;
                    }
                }
            }
            false
        });
    }

    for note in &notes {
        store.db().reindex_single_note(store.root(), note)?;
    }

    if !cli.quiet {
        println!("Indexed {} notes (selective)", notes.len());
    }

    Ok(())
}
