//! Command dispatch logic for qipu
use std::env;
use std::path::PathBuf;
use std::time::Instant;

use chrono::DateTime;

use crate::cli::{Cli, Commands, LinkCommands, OutputFormat};
use crate::commands;
use crate::lib::error::{QipuError, Result};
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

pub fn run(cli: &Cli, start: Instant) -> Result<()> {
    // Determine the root directory
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if cli.verbose {
        eprintln!("resolve_root: {:?}", start.elapsed());
    }

    // Handle commands
    match &cli.command {
        None => handle_no_command(),

        Some(Commands::Init {
            visible,
            stealth,
            branch,
        }) => handle_init(cli, &root, *stealth, *visible, branch.clone()),

        Some(Commands::Create(args)) | Some(Commands::New(args)) => {
            handle_create(cli, &root, args, start)
        }

        Some(Commands::List { tag, r#type, since }) => {
            handle_list(cli, &root, tag.as_deref(), *r#type, since.as_deref(), start)
        }

        Some(Commands::Show { id_or_path, links }) => {
            handle_show(cli, &root, id_or_path, *links, start)
        }

        Some(Commands::Inbox { exclude_linked }) => {
            handle_inbox(cli, &root, *exclude_linked, start)
        }

        Some(Commands::Capture {
            title,
            r#type,
            tag,
            source,
            author,
            generated_by,
            prompt_hash,
            verified,
            id,
        }) => handle_capture(
            cli,
            &root,
            title.as_deref(),
            *r#type,
            tag,
            source.clone(),
            author.clone(),
            generated_by.clone(),
            prompt_hash.clone(),
            *verified,
            id.as_deref(),
            start,
        ),

        Some(Commands::Index { rebuild }) => handle_index(cli, &root, *rebuild, start),

        Some(Commands::Search {
            query,
            r#type,
            tag,
            exclude_mocs,
        }) => handle_search(
            cli,
            &root,
            query,
            *r#type,
            tag.as_deref(),
            *exclude_mocs,
            start,
        ),

        Some(Commands::Verify { id_or_path, status }) => {
            handle_verify(cli, &root, id_or_path, *status, start)
        }

        Some(Commands::Prime) => handle_prime(cli, &root, start),

        Some(Commands::Setup {
            list,
            tool,
            print,
            check,
            remove,
        }) => handle_setup(cli, *list, tool.as_deref(), *print, *check, *remove),

        Some(Commands::Doctor {
            fix,
            duplicates,
            threshold,
        }) => handle_doctor(cli, &root, *fix, *duplicates, *threshold, start),

        Some(Commands::Sync {
            validate,
            fix,
            commit,
            push,
        }) => handle_sync(cli, &root, *validate, *fix, *commit, *push, start),

        Some(Commands::Context {
            note,
            tag,
            moc,
            query,
            max_chars,
            max_tokens,
            model,
            transitive,
            with_body,
            safety_banner,
        }) => handle_context(
            cli,
            &root,
            note,
            tag.as_deref(),
            moc.as_deref(),
            query.as_deref(),
            *max_chars,
            *max_tokens,
            model.as_str(),
            *transitive,
            *with_body,
            *safety_banner,
            start,
        ),

        Some(Commands::Export {
            note,
            tag,
            moc,
            query,
            output,
            mode,
            with_attachments,
            link_mode,
        }) => handle_export(
            cli,
            &root,
            note,
            tag.as_deref(),
            moc.as_deref(),
            query.as_deref(),
            output.as_ref(),
            mode,
            *with_attachments,
            link_mode,
            start,
        ),

        Some(Commands::Link { command }) => handle_link(cli, &root, command, start),

        Some(Commands::Compact { command }) => handle_compact(cli, command),

        Some(Commands::Workspace { command }) => handle_workspace(cli, command),

        Some(Commands::Dump {
            file,
            note,
            tag,
            moc,
            query,
            direction,
            max_hops,
            r#type,
            typed_only,
            inline_only,
            no_attachments,
            output,
        }) => handle_dump(
            cli,
            &root,
            file.as_ref(),
            note,
            tag.as_deref(),
            moc.as_deref(),
            query.as_deref(),
            direction,
            *max_hops,
            r#type.clone(),
            *typed_only,
            *inline_only,
            *no_attachments,
            output.as_ref(),
            start,
        ),

        Some(Commands::Load {
            pack_file,
            strategy,
        }) => handle_load(cli, &root, pack_file, strategy, start),

        Some(Commands::Merge { id1, id2, dry_run }) => {
            handle_merge(cli, &root, id1, id2, *dry_run, start)
        }
    }
}

fn discover_or_open_store(cli: &Cli, root: &PathBuf) -> Result<Store> {
    let base_store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(root)?
    };

    if let Some(workspace_name) = &cli.workspace {
        use crate::lib::store::paths::WORKSPACES_DIR;
        let workspace_path = base_store.root().join(WORKSPACES_DIR).join(workspace_name);
        Store::open(&workspace_path)
    } else {
        Ok(base_store)
    }
}

// ============================================================================
// Command Handlers
// ============================================================================

fn handle_no_command() -> Result<()> {
    println!("qipu {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A Zettelkasten-inspired knowledge management CLI.");
    println!();
    println!("Run `qipu --help` for usage information.");
    Ok(())
}

fn handle_init(
    cli: &Cli,
    root: &PathBuf,
    stealth: bool,
    visible: bool,
    branch: Option<String>,
) -> Result<()> {
    commands::init::execute(cli, root, stealth, visible, branch)
}

fn handle_create(
    cli: &Cli,
    root: &PathBuf,
    args: &crate::cli::CreateArgs,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
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
    )
}

fn handle_list(
    cli: &Cli,
    root: &PathBuf,
    tag: Option<&str>,
    note_type: Option<crate::lib::note::NoteType>,
    since: Option<&str>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }

    let since_dt = since
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| QipuError::UsageError(format!("invalid --since date '{}': {}", s, e)))
        })
        .transpose()?;

    commands::list::execute(cli, &store, tag, note_type, since_dt)
}

fn handle_show(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    links: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::show::execute(cli, &store, id_or_path, links)
}

fn handle_inbox(cli: &Cli, root: &PathBuf, exclude_linked: bool, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
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

    output_inbox_notes(cli, &store, &inbox_notes)
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
fn handle_capture(
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
        eprintln!("discover_store: {:?}", start.elapsed());
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
    )
}

fn handle_index(cli: &Cli, root: &PathBuf, rebuild: bool, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::index::execute(cli, &store, rebuild)
}

#[allow(clippy::too_many_arguments)]
fn handle_search(
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
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::search::execute(cli, &store, query, note_type, tag, exclude_mocs)
}

fn handle_verify(
    cli: &Cli,
    root: &PathBuf,
    id_or_path: &str,
    status: Option<bool>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::verify::execute(cli, &store, id_or_path, status)
}

fn handle_prime(cli: &Cli, root: &PathBuf, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::prime::execute(cli, &store)
}

fn handle_setup(
    cli: &Cli,
    list: bool,
    tool: Option<&str>,
    print: bool,
    check: bool,
    remove: bool,
) -> Result<()> {
    commands::setup::execute(cli, list, tool, print, check, remove)
}

fn handle_doctor(
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
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::doctor::execute(cli, &store, fix, duplicates, threshold)?;
    Ok(())
}

fn handle_sync(
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
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::sync::execute(cli, &store, validate, fix, commit, push)
}

#[allow(clippy::too_many_arguments)]
fn handle_context(
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
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
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
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_export(
    cli: &Cli,
    root: &PathBuf,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    output: Option<&PathBuf>,
    mode: &str,
    with_attachments: bool,
    link_mode: &str,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    let export_mode = commands::export::ExportMode::parse(mode)?;
    let link_mode = commands::export::LinkMode::parse(link_mode)?;
    commands::export::execute(
        cli,
        &store,
        commands::export::ExportOptions {
            note_ids,
            tag,
            moc_id,
            query,
            output: output.map(|p| p.as_path()),
            mode: export_mode,
            with_attachments,
            link_mode,
        },
    )
}

fn handle_link(cli: &Cli, root: &PathBuf, command: &LinkCommands, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }

    match command {
        LinkCommands::List {
            id_or_path,
            direction,
            r#type,
            typed_only,
            inline_only,
            max_chars,
        } => {
            let dir = direction
                .parse::<crate::lib::graph::Direction>()
                .map_err(QipuError::Other)?;
            commands::link::list::execute(
                cli,
                &store,
                id_or_path,
                dir,
                r#type.as_deref(),
                *typed_only,
                *inline_only,
                *max_chars,
            )
        }
        LinkCommands::Add { from, to, r#type } => {
            commands::link::add::execute(cli, &store, from, to, r#type.clone())
        }
        LinkCommands::Remove { from, to, r#type } => {
            commands::link::remove::execute(cli, &store, from, to, r#type.clone())
        }
        LinkCommands::Tree {
            id_or_path,
            direction,
            max_hops,
            r#type,
            exclude_type,
            typed_only,
            inline_only,
            max_nodes,
            max_edges,
            max_fanout,
            max_chars,
        } => {
            let dir = direction
                .parse::<crate::lib::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = crate::lib::graph::TreeOptions {
                direction: dir,
                max_hops: *max_hops,
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: *max_nodes,
                max_edges: *max_edges,
                max_fanout: *max_fanout,
                max_chars: *max_chars,
                semantic_inversion: true,
            };
            commands::link::tree::execute(cli, &store, id_or_path, opts)
        }
        LinkCommands::Path {
            from,
            to,
            direction,
            max_hops,
            r#type,
            exclude_type,
            typed_only,
            inline_only,
            max_chars,
        } => {
            let dir = direction
                .parse::<crate::lib::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = crate::lib::graph::TreeOptions {
                direction: dir,
                max_hops: *max_hops,
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: None,
                max_edges: None,
                max_fanout: None,
                max_chars: *max_chars,
                semantic_inversion: true,
            };
            commands::link::path::execute(cli, &store, from, to, opts)
        }
    }
}

fn handle_compact(cli: &Cli, command: &crate::cli::CompactCommands) -> Result<()> {
    commands::compact::execute(cli, command)
}

fn handle_workspace(cli: &Cli, command: &crate::cli::WorkspaceCommands) -> Result<()> {
    commands::workspace::execute(cli, command)
}

#[allow(clippy::too_many_arguments)]
fn handle_dump(
    cli: &Cli,
    root: &PathBuf,
    file: Option<&PathBuf>,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    direction: &str,
    max_hops: u32,
    type_include: Vec<String>,
    typed_only: bool,
    inline_only: bool,
    no_attachments: bool,
    output: Option<&PathBuf>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }

    let dir = direction
        .parse::<commands::link::Direction>()
        .map_err(QipuError::Other)?;

    let resolved_output = match (file, output) {
        (Some(_), Some(_)) => {
            return Err(QipuError::Other(
                "both positional file and --output were provided; use one".to_string(),
            ))
        }
        (Some(file_path), None) => Some(file_path.as_path()),
        (None, Some(output_path)) => Some(output_path.as_path()),
        (None, None) => None,
    };

    commands::dump::execute(
        cli,
        &store,
        commands::dump::DumpOptions {
            note_ids,
            tag,
            moc_id,
            query,
            direction: dir,
            max_hops,
            type_include,
            typed_only,
            inline_only,
            include_attachments: !no_attachments,
            output: resolved_output,
        },
    )
}

fn handle_load(
    cli: &Cli,
    root: &PathBuf,
    pack_file: &PathBuf,
    strategy: &str,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::load::execute(cli, &store, pack_file, strategy)
}

fn handle_merge(
    cli: &Cli,
    root: &PathBuf,
    id1: &str,
    id2: &str,
    dry_run: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        eprintln!("discover_store: {:?}", start.elapsed());
    }
    commands::merge::execute(cli, &store, id1, id2, dry_run)
}
