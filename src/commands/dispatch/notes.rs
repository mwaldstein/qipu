//! Handlers for note-related commands

use std::path::PathBuf;
use std::time::Instant;

use chrono::DateTime;
use tracing::debug;

use crate::cli::{Cli, OutputFormat};
use crate::commands;
use qipu_core::error::{QipuError, Result};
use qipu_core::records::escape_quotes;
use qipu_core::store::Store;

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
        args.r#type.clone(),
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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_list(
    cli: &Cli,
    root: &PathBuf,
    tag: Option<&str>,
    note_type: Option<qipu_core::note::NoteType>,
    since: Option<&str>,
    min_value: Option<u8>,
    custom: Option<&str>,
    show_custom: bool,
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

    commands::list::execute(
        cli,
        &store,
        tag,
        note_type,
        since_dt,
        min_value,
        custom,
        show_custom,
    )?;
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
    show_custom: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::show::execute(cli, &store, id_or_path, links, show_custom)?;
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
        let compaction_ctx = qipu_core::compaction::CompactionContext::build(&notes)?;
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
                n.note_type().as_str(),
                qipu_core::note::NoteType::FLEETING | qipu_core::note::NoteType::LITERATURE
            )
        })
        .collect();

    // Filter out notes linked from MOCs if requested
    if exclude_linked {
        let index = qipu_core::index::IndexBuilder::new(&store).build()?;
        if cli.verbose {
            debug!(elapsed = ?start.elapsed(), "load_indexes");
        }
        let mut linked_from_mocs = std::collections::HashSet::new();
        for edge in &index.edges {
            if let Some(source_meta) = index.get_metadata(&edge.from) {
                if source_meta.note_type.is_moc() {
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
    inbox_notes: &[qipu_core::note::Note],
) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = inbox_notes
                .iter()
                .map(|n| {
                    let mut obj = serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                        "created": n.frontmatter.created,
                        "updated": n.frontmatter.updated,
                    });

                    if let Some(path) = &n.path {
                        if let Some(obj_mut) = obj.as_object_mut() {
                            obj_mut.insert(
                                "path".to_string(),
                                serde_json::json!(path.to_string_lossy()),
                            );
                        }
                    }

                    obj
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
                    let type_indicator = match note.note_type().as_str() {
                        qipu_core::note::NoteType::FLEETING => "F",
                        qipu_core::note::NoteType::LITERATURE => "L",
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
    note_type: Option<qipu_core::note::NoteType>,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_search(
    cli: &Cli,
    root: &PathBuf,
    query: &str,
    note_type: Option<qipu_core::note::NoteType>,
    tag: Option<&str>,
    exclude_mocs: bool,
    min_value: Option<u8>,
    sort: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::search::execute(
        cli,
        &store,
        query,
        note_type,
        tag,
        exclude_mocs,
        min_value,
        sort,
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_context(
    cli: &Cli,
    root: &PathBuf,
    walk_id: Option<&str>,
    walk_direction: &str,
    walk_max_hops: u32,
    walk_type: &[String],
    walk_exclude_type: &[String],
    walk_typed_only: bool,
    walk_inline_only: bool,
    walk_max_nodes: Option<usize>,
    walk_max_edges: Option<usize>,
    walk_max_fanout: Option<usize>,
    walk_min_value: Option<u8>,
    walk_ignore_value: bool,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    max_chars: Option<usize>,
    transitive: bool,
    with_body: bool,
    safety_banner: bool,
    related: f64,
    backlinks: bool,
    min_value: Option<u8>,
    custom_filter: &[String],
    include_custom: bool,
    include_ontology: bool,
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
            walk_id,
            walk_direction,
            walk_max_hops,
            walk_type,
            walk_exclude_type,
            walk_typed_only,
            walk_inline_only,
            walk_max_nodes,
            walk_max_edges,
            walk_max_fanout,
            walk_min_value,
            walk_ignore_value,
            note_ids,
            tag,
            moc_id,
            query,
            max_chars,
            transitive,
            with_body,
            safety_banner,
            related_threshold: if related > 0.0 { Some(related) } else { None },
            backlinks,
            min_value,
            custom_filter,
            include_custom,
            include_ontology,
        },
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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

pub(super) fn handle_edit(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    editor: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::edit::execute(cli, &store, id_or_path, editor)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_update(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    title: Option<&str>,
    note_type: Option<qipu_core::note::NoteType>,
    tags: &[String],
    remove_tags: &[String],
    value: Option<u8>,
    source: Option<&str>,
    author: Option<&str>,
    generated_by: Option<&str>,
    prompt_hash: Option<&str>,
    verified: Option<bool>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::update::execute(
        cli,
        &store,
        id_or_path,
        title,
        note_type,
        tags,
        remove_tags,
        value,
        source,
        author,
        generated_by,
        prompt_hash,
        verified,
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}
