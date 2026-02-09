//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate
//!
#![allow(clippy::if_same_then_else)]

use crate::cli::Cli;
use crate::commands::format::{dispatch_format, FormatDispatcher};
use qipu_core::error::{QipuError, Result};
use qipu_core::index::links;
use qipu_core::note::{Note, NoteType};
use qipu_core::store::Store;
use std::time::Instant;

struct IndexFormatter<'a> {
    store: &'a Store,
    notes_count: usize,
}

impl<'a> FormatDispatcher for IndexFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "status": "ok",
            "notes_indexed": self.notes_count,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("Indexed {} notes", self.notes_count);
    }

    fn output_records(&self) {
        println!(
            "H qipu=1 records=1 store={} mode=index notes={}",
            self.store.root().display(),
            self.notes_count
        );
    }
}

struct IndexStatusFormatter<'a> {
    store: &'a Store,
    db_count: i64,
    basic_count: i64,
    full_count: i64,
}

impl<'a> FormatDispatcher for IndexStatusFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "total_notes": self.db_count,
            "basic_indexed": self.basic_count,
            "full_indexed": self.full_count,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("Index Status");
        println!("-------------");
        println!("Total notes: {}", self.db_count);
        println!(
            "Basic indexed: {} ({})",
            self.basic_count,
            if self.db_count > 0 {
                format!(
                    "{:.0}%",
                    (self.basic_count as f64) / (self.db_count as f64) * 100.0
                )
            } else {
                "N/A".to_string()
            }
        );
        println!(
            "Full-text indexed: {} ({})",
            self.full_count,
            if self.db_count > 0 {
                format!(
                    "{:.0}%",
                    (self.full_count as f64) / (self.db_count as f64) * 100.0
                )
            } else {
                "N/A".to_string()
            }
        );
    }

    fn output_records(&self) {
        println!(
            "H qipu=1 records=1 store={} mode=status total={} basic={} full={}",
            self.store.root().display(),
            self.db_count,
            self.basic_count,
            self.full_count
        );
    }
}

/// Progress tracker for indexing operations
struct ProgressTracker {
    first_update_time: Option<Instant>,
    last_update_time: Option<Instant>,
    last_indexed: usize,
    notes_per_sec: f64,
}

impl ProgressTracker {
    fn new() -> Self {
        Self {
            first_update_time: None,
            last_update_time: None,
            last_indexed: 0,
            notes_per_sec: 0.0,
        }
    }

    fn update(&mut self, indexed: usize, total: usize, note: &Note) {
        let now = Instant::now();

        if self.first_update_time.is_none() {
            self.first_update_time = Some(now);
        }

        if let Some(last_time) = self.last_update_time {
            let elapsed = now.duration_since(last_time).as_secs_f64();
            let indexed_delta = indexed - self.last_indexed;

            if elapsed > 0.0 && indexed_delta > 0 {
                self.notes_per_sec = indexed_delta as f64 / elapsed;
            }
        }

        self.last_update_time = Some(now);
        self.last_indexed = indexed;

        let percent = (indexed as f64 / total as f64) * 100.0;
        let remaining = total - indexed;

        let eta_str = if self.notes_per_sec > 0.0 {
            let eta_secs = remaining as f64 / self.notes_per_sec;
            if eta_secs < 1.0 {
                "1s".to_string()
            } else if eta_secs < 60.0 {
                format!("{:.0}s", eta_secs.ceil())
            } else {
                format!("{:.0}m {:.0}s", (eta_secs / 60.0).floor(), eta_secs % 60.0)
            }
        } else {
            "---".to_string()
        };

        let bar_width = 30;
        let filled = (bar_width as f64 * percent / 100.0) as usize;
        let filled = filled.min(bar_width);
        let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);

        eprintln!(
            "  [{}] {:.0}% ({} / {}) {:.0} notes/sec",
            bar, percent, indexed, total, self.notes_per_sec
        );
        eprintln!(
            "  ETA: {}  Last: {} \"{}\"",
            eta_str,
            note.id(),
            note.title()
        );
    }
}

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

/// Parse a time string like "24 hours ago", "2 days ago", "1 week ago", or ISO 8601 timestamp
fn parse_modified_since(s: &str) -> Result<std::time::SystemTime> {
    use std::time::{Duration, SystemTime};

    let now = SystemTime::now();
    let s_lower = s.to_lowercase();

    // Try to parse relative time expressions
    if s_lower.ends_with(" ago") {
        let parts: Vec<&str> = s_lower
            .trim_end_matches(" ago")
            .split_whitespace()
            .collect();
        if parts.len() == 2 {
            if let Ok(amount) = parts[0].parse::<u64>() {
                let duration = match parts[1] {
                    "second" | "seconds" => Duration::from_secs(amount),
                    "minute" | "minutes" => Duration::from_secs(amount * 60),
                    "hour" | "hours" => Duration::from_secs(amount * 60 * 60),
                    "day" | "days" => Duration::from_secs(amount * 24 * 60 * 60),
                    "week" | "weeks" => Duration::from_secs(amount * 7 * 24 * 60 * 60),
                    _ => {
                        return Err(QipuError::Other(format!(
                            "Unknown time unit: {}. Use seconds, minutes, hours, days, or weeks",
                            parts[1]
                        )));
                    }
                };
                return now.checked_sub(duration).ok_or_else(|| {
                    QipuError::Other("Invalid time duration: too far in the past".to_string())
                });
            }
        }
    }

    // Try ISO 8601 format
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(datetime.into());
    }

    // Try simpler ISO format (2024-01-15T10:30:00)
    if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(
            SystemTime::UNIX_EPOCH + Duration::from_secs(datetime.and_utc().timestamp() as u64)
        );
    }

    // Try date-only format (2024-01-15)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(
            SystemTime::UNIX_EPOCH + Duration::from_secs(datetime.and_utc().timestamp() as u64)
        );
    }

    Err(QipuError::Other(format!(
        "Cannot parse time: '{}'. Use formats like '24 hours ago', '2 days ago', '2024-01-15', or ISO 8601",
        s
    )))
}

fn filter_quick_index(
    _store: &Store,
    notes: &[qipu_core::note::Note],
) -> Vec<qipu_core::note::Note> {
    let mut mocs = Vec::new();
    let mut others: Vec<(std::time::SystemTime, qipu_core::note::Note)> = Vec::new();

    for note in notes {
        if note.note_type().is_moc() {
            mocs.push(note.clone());
        } else if let Some(path) = &note.path {
            if let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) {
                others.push((mtime, note.clone()));
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
    notes: &[qipu_core::note::Note],
    moc_id: &str,
) -> Vec<qipu_core::note::Note> {
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

fn filter_by_recent(notes: &[qipu_core::note::Note], n: usize) -> Vec<qipu_core::note::Note> {
    let mut notes_with_mtime: Vec<(std::time::SystemTime, qipu_core::note::Note)> = Vec::new();

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
