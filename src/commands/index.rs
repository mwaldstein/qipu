//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate
//!
#![allow(clippy::if_same_then_else)]

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::index::links;
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Execute index command
pub fn execute(
    cli: &Cli,
    store: &Store,
    rebuild: bool,
    rewrite_wiki_links: bool,
    quick: bool,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    recent: Option<usize>,
    moc: Option<&str>,
    status: bool,
) -> Result<()> {
    if status {
        return show_index_status(cli, store);
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

    if cli.verbose {
        let progress = |indexed: usize, total: usize, last_id: &str| {
            eprintln!(
                "Indexing: {}/{} notes ({:.0}%) - Last: {}",
                indexed,
                total,
                (indexed as f64 / total as f64) * 100.0,
                last_id
            );
        };

        if quick || tag.is_some() || note_type.is_some() || recent.is_some() || moc.is_some() {
            selective_index(cli, store, quick, tag, note_type, recent, moc)?;
        } else if rebuild {
            store.db().rebuild(store.root(), Some(&progress))?;
        } else {
            store
                .db()
                .incremental_repair(store.root(), Some(&progress))?;
        }
    } else {
        if quick || tag.is_some() || note_type.is_some() || recent.is_some() || moc.is_some() {
            selective_index(cli, store, quick, tag, note_type, recent, moc)?;
        } else if rebuild {
            store.db().rebuild(store.root(), None)?;
        } else {
            store.db().incremental_repair(store.root(), None)?;
        }
    }

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "ok",
                "notes_indexed": notes_count,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            let store_path = store.root().display();

            println!(
                "H qipu=1 records=1 store={} mode=index notes={}",
                store_path, notes_count
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!("Indexed {} notes", notes_count);
            }
        }
    }

    Ok(())
}

fn show_index_status(cli: &Cli, store: &Store) -> Result<()> {
    let db_count = store.db().get_note_count().unwrap_or(0);
    let basic_count = store.db().count_basic_indexed().unwrap_or(0);
    let full_count = store.db().count_full_indexed().unwrap_or(0);

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "total_notes": db_count,
                "basic_indexed": basic_count,
                "full_indexed": full_count,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=status total={} basic={} full={}",
                store.root().display(),
                db_count,
                basic_count,
                full_count
            );
        }
        OutputFormat::Human => {
            println!("Index Status");
            println!("-------------");
            println!("Total notes: {}", db_count);
            println!(
                "Basic indexed: {} ({})",
                basic_count,
                if db_count > 0 {
                    format!("{:.0}%", basic_count as f64 / db_count as f64 * 100.0)
                } else {
                    "N/A".to_string()
                }
            );
            println!(
                "Full-text indexed: {} ({})",
                full_count,
                if db_count > 0 {
                    format!("{:.0}%", full_count as f64 / db_count as f64 * 100.0)
                } else {
                    "N/A".to_string()
                }
            );
        }
    }

    Ok(())
}

fn selective_index(
    cli: &Cli,
    store: &Store,
    quick: bool,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    recent: Option<usize>,
    moc: Option<&str>,
) -> Result<()> {
    let mut notes = store.list_notes()?;

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

    for note in &notes {
        store.db().reindex_single_note(store.root(), note)?;
    }

    if !cli.quiet {
        println!("Indexed {} notes (selective)", notes.len());
    }

    Ok(())
}

fn filter_quick_index(
    _store: &Store,
    notes: &[crate::lib::note::Note],
) -> Vec<crate::lib::note::Note> {
    use crate::lib::note::NoteType;
    let mut mocs = Vec::new();
    let mut others: Vec<(std::time::SystemTime, crate::lib::note::Note)> = Vec::new();

    for note in notes {
        if note.note_type() == NoteType::Moc {
            mocs.push(note.clone());
        } else {
            if let Some(path) = &note.path {
                if let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) {
                    others.push((mtime, note.clone()));
                }
            }
        }
    }

    others.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = mocs;
    for (_, note) in others.into_iter().take(100) {
        result.push(note);
    }

    result
}

fn filter_by_moc(
    store: &Store,
    notes: &[crate::lib::note::Note],
    moc_id: &str,
) -> Vec<crate::lib::note::Note> {
    let mut result = Vec::new();

    let moc = notes.iter().find(|n| n.id() == moc_id);
    if let Some(m) = moc {
        result.push(m.clone());

        let outbound_edges = store.db().get_outbound_edges(moc_id).unwrap_or_default();
        for edge in outbound_edges {
            if let Some(note) = notes.iter().find(|n| n.id() == edge.to) {
                result.push(note.clone());
            }
        }
    }

    result
}

fn filter_by_recent(notes: &[crate::lib::note::Note], n: usize) -> Vec<crate::lib::note::Note> {
    let mut notes_with_mtime: Vec<(std::time::SystemTime, crate::lib::note::Note)> = Vec::new();

    for note in notes {
        if let Some(path) = &note.path {
            if let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) {
                notes_with_mtime.push((mtime, note.clone()));
            }
        }
    }

    notes_with_mtime.sort_by(|a, b| b.0.cmp(&a.0));
    notes_with_mtime
        .into_iter()
        .take(n)
        .map(|(_, note)| note)
        .collect()
}
