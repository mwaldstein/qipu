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
    let store = match discover_or_open_store(cli, root) {
        Ok(store) => store,
        Err(_) => {
            // For doctor, try unchecked open if discovery fails
            let qipu_path = root.join(".qipu");
            if qipu_path.is_dir() {
                Store::open_unchecked(&qipu_path)?
            } else {
                let visible_path = root.join("qipu");
                if visible_path.is_dir() {
                    Store::open_unchecked(&visible_path)?
                } else {
                    return Err(QipuError::StoreNotFound {
                        search_root: root.clone(),
                    });
                }
            }
        }
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

pub(super) fn handle_index(cli: &Cli, root: &PathBuf, rebuild: bool, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::index::execute(cli, &store, rebuild)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_prime(cli: &Cli, root: &PathBuf, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::prime::execute(cli, &store)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}
