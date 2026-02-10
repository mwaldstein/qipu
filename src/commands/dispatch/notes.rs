//! Handlers for note-related commands

use std::path::Path;
use std::time::Instant;

use chrono::DateTime;

use crate::cli::{Cli, OutputFormat};
use crate::commands;
use qipu_core::error::{QipuError, Result};
use qipu_core::records::escape_quotes;
use qipu_core::store::Store;

use super::command::discover_or_open_store;
#[allow(unused_imports)]
use super::trace_command;

pub(super) fn handle_create(
    cli: &Cli,
    root: &Path,
    args: &crate::cli::CreateArgs,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::create::execute(
        cli,
        &store,
        &args.title,
        args.r#type.as_ref(),
        &args.tag,
        args.open,
        args.id.as_deref(),
        args.source.as_deref(),
        args.author.as_deref(),
        args.generated_by.as_deref(),
        args.prompt_hash.as_deref(),
        args.verified,
    )?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

pub struct ListOptions<'a> {
    pub tag: Option<&'a str>,
    pub note_type: Option<qipu_core::note::NoteType>,
    pub since: Option<&'a str>,
    pub min_value: Option<u8>,
    pub custom: Option<&'a str>,
    pub show_custom: bool,
    pub start: Instant,
}

pub(super) fn handle_list(cli: &Cli, root: &Path, opts: ListOptions<'_>) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, opts.start, "discover_store");

    let since_dt = opts
        .since
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| QipuError::UsageError(format!("invalid --since date '{}': {}", s, e)))
        })
        .transpose()?;

    commands::list::execute(
        cli,
        &store,
        opts.tag,
        opts.note_type,
        since_dt,
        opts.min_value,
        opts.custom,
        opts.show_custom,
    )?;
    trace_command!(cli, opts.start, "execute_command");
    Ok(())
}

pub(super) fn handle_show(
    cli: &Cli,
    root: &Path,
    id_or_path: &str,
    links: bool,
    show_custom: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::show::execute(cli, &store, id_or_path, links, show_custom)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

pub(super) fn handle_inbox(
    cli: &Cli,
    root: &Path,
    exclude_linked: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");

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
        trace_command!(cli, start, "load_indexes");
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
    trace_command!(cli, start, "execute_command");
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
                let tags_csv = note.frontmatter.format_tags();
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

pub struct CaptureOptions<'a> {
    pub title: Option<&'a str>,
    pub note_type: Option<&'a qipu_core::note::NoteType>,
    pub tags: &'a [String],
    pub source: Option<&'a str>,
    pub author: Option<&'a str>,
    pub generated_by: Option<&'a str>,
    pub prompt_hash: Option<&'a str>,
    pub verified: Option<bool>,
    pub id: Option<&'a str>,
    pub start: Instant,
}

pub(super) fn handle_capture<'a>(cli: &Cli, root: &Path, opts: CaptureOptions<'a>) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, opts.start, "discover_store");
    commands::capture::execute(
        cli,
        &store,
        opts.title,
        opts.note_type,
        opts.tags,
        opts.source,
        opts.author,
        opts.generated_by,
        opts.prompt_hash,
        opts.verified,
        opts.id,
    )?;
    trace_command!(cli, opts.start, "execute_command");
    Ok(())
}

pub(super) fn execute_capture_from_args(
    cli: &Cli,
    root: &Path,
    args: &crate::cli::commands::core::CaptureArgs,
    start: Instant,
) -> Result<()> {
    handle_capture(
        cli,
        root,
        CaptureOptions {
            title: args.title.as_deref(),
            note_type: args.r#type.as_ref(),
            tags: &args.tag,
            source: args.source.as_deref(),
            author: args.author.as_deref(),
            generated_by: args.generated_by.as_deref(),
            prompt_hash: args.prompt_hash.as_deref(),
            verified: args.verified,
            id: args.id.as_deref(),
            start,
        },
    )
}

pub(super) fn execute_update_from_args(
    cli: &Cli,
    root: &Path,
    args: &crate::cli::commands::core::UpdateArgs,
    start: Instant,
) -> Result<()> {
    handle_update(
        cli,
        root,
        &args.id_or_path,
        args.title.as_deref(),
        args.r#type.as_ref(),
        &args.tag,
        &args.remove_tag,
        args.value,
        args.source.as_deref(),
        args.author.as_deref(),
        args.generated_by.as_deref(),
        args.prompt_hash.as_deref(),
        args.verified,
        start,
    )
}

pub(super) fn handle_verify(
    cli: &Cli,
    root: &Path,
    id_or_path: &str,
    status: Option<bool>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::verify::execute(cli, &store, id_or_path, status)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_search(
    cli: &Cli,
    root: &Path,
    query: &str,
    note_type: Option<qipu_core::note::NoteType>,
    tag: Option<&str>,
    exclude_mocs: bool,
    min_value: Option<u8>,
    sort: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
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
    trace_command!(cli, start, "execute_command");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_merge(
    cli: &Cli,
    root: &Path,
    id1: &str,
    id2: &str,
    dry_run: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::merge::execute(cli, &store, id1, id2, dry_run)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

pub(super) fn handle_edit(
    cli: &Cli,
    root: &Path,
    id_or_path: &str,
    editor: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");
    commands::edit::execute(cli, &store, id_or_path, editor)?;
    trace_command!(cli, start, "execute_command");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_update(
    cli: &Cli,
    root: &Path,
    id_or_path: &str,
    title: Option<&str>,
    note_type: Option<&qipu_core::note::NoteType>,
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
    trace_command!(cli, start, "discover_store");
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
    trace_command!(cli, start, "execute_command");
    Ok(())
}
