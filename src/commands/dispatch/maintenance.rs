//! Handlers for maintenance commands (doctor, sync, index, prime)

use std::path::Path;
use std::time::Instant;

use crate::cli::Cli;
use crate::commands;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

use super::command::discover_or_open_store;
#[allow(unused_imports)]
use super::trace_command;

pub(super) fn handle_doctor(
    cli: &Cli,
    root: &Path,
    fix: bool,
    duplicates: bool,
    threshold: f64,
    check: Option<&[String]>,
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
            search_root: root.to_path_buf(),
        });
    };

    trace_command!(cli, start, "discover_store");
    let check_ontology = check.is_some_and(|checks| checks.contains(&"ontology".to_string()));
    commands::doctor::execute(cli, &store, fix, duplicates, threshold, check_ontology)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

pub(super) fn handle_sync(
    cli: &Cli,
    root: &Path,
    validate: bool,
    fix: bool,
    commit: bool,
    push: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::sync::execute(cli, &store, validate, fix, commit, push)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_index(
    cli: &Cli,
    root: &Path,
    rebuild: bool,
    resume: bool,
    rewrite_wiki_links: bool,
    quick: bool,
    tag: Option<String>,
    note_type: Option<qipu_core::note::NoteType>,
    recent: Option<usize>,
    moc: Option<String>,
    status: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
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
    trace_command!(cli, start, "execute_command");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_prime(
    cli: &Cli,
    root: &Path,
    compact: bool,
    minimal: bool,
    full: bool,
    mcp: bool,
    use_prime_md: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::prime::execute(cli, &store, compact, minimal, full, mcp, use_prime_md)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}
