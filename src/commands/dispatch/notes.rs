//! Handlers for note-related commands

use std::path::PathBuf;
use std::time::Instant;

use chrono::DateTime;
use tracing::debug;

use crate::cli::{Cli, OutputFormat};
use crate::commands;
use crate::lib::error::{QipuError, Result};
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

use super::discover_or_open_store;

pub(super) fn handle_create(
    cli: &Cli,
    root: &PathBuf,
    args: &crate::cli::CreateArgs,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::create::execute(
        cli,
        &store,
        &args.title,
        args.r#type,
        &args.tag,
        args.open,
        args.id.clone(),
        args.source.clone(),
        args.author.clone(),
        args.generated_by.clone(),
        args.prompt_hash.clone(),
        args.verified,
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_list(
    cli: &Cli,
    root: &PathBuf,
    tag: Option<&str>,
    note_type: Option<crate::lib::note::NoteType>,
    since: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }

    let since_dt = since
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| QipuError::UsageError(format!("invalid --since date '{}': {}", s, e)))
        })
        .transpose()?;

    commands::list::execute(cli, &store, tag, note_type, since_dt)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_show(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    links: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::show::execute(cli, &store, id_or_path, links)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_inbox(
    cli: &Cli,
    root: &PathBuf,
    exclude_linked: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }

    let notes = store.list_notes()?;

    // Apply compaction visibility filter
    let notes = if !cli.no_resolve_compaction {
        let compaction_ctx = crate::lib::compaction::CompactionContext::build(&notes)?;
        notes
            .into_iter()
            .filter(|n| !compaction_ctx.is_compacted(&n.frontmatter.id))
            .collect()
    } else {
        notes
    };

    let mut inbox_notes: Vec<_> = notes
        .into_iter()
        .filter(|n| {
            matches!(
                n.note_type(),
                crate::lib::note::NoteType::Fleeting | crate::lib::note::NoteType::Literature
            )
        })
        .collect();

    // Filter out notes linked from MOCs if requested
    if exclude_linked {
        let index = crate::lib::index::IndexBuilder::new(&store).build()?;
        if cli.verbose {
            debug!(elapsed = ?start.elapsed(), "load_indexes");
        }
        let mut linked_from_mocs = std::collections::HashSet::new();
        for edge in &index.edges {
            if let Some(source_meta) = index.get_metadata(&edge.from) {
                if source_meta.note_type == crate::lib::note::NoteType::Moc {
                    linked_from_mocs.insert(edge.to.clone());
                }
            }
        }
        inbox_notes.retain(|n| !linked_from_mocs.contains(n.id()));
    }

    output_inbox_notes(cli, &store, &inbox_notes)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

fn output_inbox_notes(
    cli: &Cli,
    store: &Store,
    inbox_notes: &[crate::lib::note::Note],
) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = inbox_notes
                .iter()
                .map(|n| {
                    serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                        "path": n.path.as_ref().map(|p| p.display().to_string()),
                        "created": n.frontmatter.created,
                        "updated": n.frontmatter.updated,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if inbox_notes.is_empty() {
                if !cli.quiet {
                    println!("Inbox is empty");
                }
            } else {
                for note in inbox_notes {
                    let type_indicator = match note.note_type() {
                        crate::lib::note::NoteType::Fleeting => "F",
                        crate::lib::note::NoteType::Literature => "L",
                        _ => "?",
                    };
                    println!("{} [{}] {}", note.id(), type_indicator, note.title());
                }
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=inbox notes={}",
                store.root().display(),
                inbox_notes.len()
            );
            for note in inbox_notes {
                let tags_csv = if note.frontmatter.tags.is_empty() {
                    "-".to_string()
                } else {
                    note.frontmatter.tags.join(",")
                };
                println!(
                    "N {} {} \"{}\" tags={}",
                    note.id(),
                    note.note_type(),
                    escape_quotes(note.title()),
                    tags_csv
                );
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_capture(
    cli: &Cli,
    root: &PathBuf,
    title: Option<&str>,
    note_type: Option<crate::lib::note::NoteType>,
    tags: &[String],
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
    id: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::capture::execute(
        cli,
        &store,
        title,
        note_type,
        tags,
        source,
        author,
        generated_by,
        prompt_hash,
        verified,
        id,
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_verify(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    status: Option<bool>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::verify::execute(cli, &store, id_or_path, status)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_search(
    cli: &Cli,
    root: &PathBuf,
    query: &str,
    note_type: Option<crate::lib::note::NoteType>,
    tag: Option<&str>,
    exclude_mocs: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::search::execute(cli, &store, query, note_type, tag, exclude_mocs)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_context(
    cli: &Cli,
    root: &PathBuf,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    max_chars: Option<usize>,
    max_tokens: Option<usize>,
    model: &str,
    transitive: bool,
    with_body: bool,
    safety_banner: bool,
    related: Option<f64>,
    backlinks: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::context::execute(
        cli,
        &store,
        commands::context::ContextOptions {
            note_ids,
            tag,
            moc_id,
            query,
            max_chars,
            max_tokens,
            model,
            transitive,
            with_body,
            safety_banner,
            related_threshold: related,
            backlinks,
        },
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_merge(
    cli: &Cli,
    root: &PathBuf,
    id1: &str,
    id2: &str,
    dry_run: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::merge::execute(cli, &store, id1, id2, dry_run)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}
