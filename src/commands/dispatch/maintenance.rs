//! Handlers for maintenance commands (doctor, sync, index, prime)

use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use crate::commands;
use crate::lib::error::{QipuError, Result};
use crate::lib::store::Store;

use super::discover_or_open_store;

pub(super) fn handle_doctor(
    cli: &Cli,
    root: &PathBuf,
    fix: bool,
    duplicates: bool,
    threshold: f64,
    start: Instant,
) -> Result<()> {
    // For doctor, always use unchecked open to avoid auto-repair
    // We want to detect issues, not fix them automatically
    let qipu_path = root.join(".qipu");
    let visible_path = root.join("qipu");
    let store = if qipu_path.is_dir() {
        Store::open_unchecked(&qipu_path, false)?
    } else if visible_path.is_dir() {
        Store::open_unchecked(&visible_path, false)?
    } else {
        return Err(QipuError::StoreNotFound {
            search_root: root.clone(),
        });
    };

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::doctor::execute(cli, &store, fix, duplicates, threshold)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_sync(
    cli: &Cli,
    root: &PathBuf,
    validate: bool,
    fix: bool,
    commit: bool,
    push: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::sync::execute(cli, &store, validate, fix, commit, push)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_index(
    cli: &Cli,
    root: &PathBuf,
    rebuild: bool,
    resume: bool,
    rewrite_wiki_links: bool,
    quick: bool,
    tag: Option<String>,
    note_type: Option<crate::lib::note::NoteType>,
    recent: Option<usize>,
    moc: Option<String>,
    status: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::index::execute(
        cli,
        &store,
        rebuild,
        resume,
        rewrite_wiki_links,
        quick,
        tag.as_deref(),
        note_type,
        recent,
        moc.as_deref(),
        status,
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_prime(
    cli: &Cli,
    root: &PathBuf,
    compact: bool,
    minimal: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::prime::execute(cli, &store, compact, minimal)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}
