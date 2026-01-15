//! Command dispatch logic for qipu
use std::env;
use std::path::PathBuf;
use std::time::Instant;

use chrono::DateTime;

use crate::cli::{Cli, Commands, LinkCommands, OutputFormat};
use crate::commands;
use crate::lib::error::{QipuError, Result};
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
        None => {
            // No subcommand - show help
            // clap handles this automatically with --help, but we can provide a hint
            println!("qipu {}", env!("CARGO_PKG_VERSION"));
            println!();
            println!("A Zettelkasten-inspired knowledge management CLI.");
            println!();
            println!("Run `qipu --help` for usage information.");
            Ok(())
        }

        Some(Commands::Init {
            visible,
            stealth,
            branch,
        }) => commands::init::execute(cli, &root, *stealth, *visible, branch.clone()),

        Some(Commands::Create(args)) | Some(Commands::New(args)) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::create::execute(cli, &store, &args.title, args.r#type, &args.tag, args.open)
        }

        Some(Commands::List { tag, r#type, since }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            // Parse since date if provided
            let since_dt = since
                .as_ref()
                .map(|s| {
                    DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .map_err(|e| {
                            QipuError::Other(format!("invalid --since date '{}': {}", s, e))
                        })
                })
                .transpose()?;

            commands::list::execute(cli, &store, tag.as_deref(), *r#type, since_dt)
        }

        Some(Commands::Show { id_or_path, links }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::show::execute(cli, &store, id_or_path, *links)
        }

        Some(Commands::Inbox { exclude_linked }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            // Inbox is essentially list with type filter for fleeting/literature
            let notes = store.list_notes()?;

            // Apply compaction visibility filter (unless --no-resolve-compaction)
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
                        crate::lib::note::NoteType::Fleeting
                            | crate::lib::note::NoteType::Literature
                    )
                })
                .collect();

            // If --exclude-linked is specified, filter out notes linked from any MOC
            if *exclude_linked {
                let index = crate::lib::index::IndexBuilder::new(&store).build()?;

                // Build a set of note IDs that are linked from MOCs
                let mut linked_from_mocs = std::collections::HashSet::new();
                for edge in &index.edges {
                    // Check if the source note is a MOC
                    if let Some(source_meta) = index.get_metadata(&edge.from) {
                        if source_meta.note_type == crate::lib::note::NoteType::Moc {
                            linked_from_mocs.insert(edge.to.clone());
                        }
                    }
                }

                // Filter out notes that are linked from MOCs
                inbox_notes.retain(|n| !linked_from_mocs.contains(n.id()));
            }

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
                        for note in &inbox_notes {
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
                    for note in &inbox_notes {
                        let tags_csv = if note.frontmatter.tags.is_empty() {
                            "-".to_string()
                        } else {
                            note.frontmatter.tags.join(",")
                        };
                        println!(
                            "N {} {} \"{}\" tags={}",
                            note.id(),
                            note.note_type(),
                            note.title(),
                            tags_csv
                        );
                    }
                }
            }

            Ok(())
        }

        Some(Commands::Capture { title, r#type, tag }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::capture::execute(cli, &store, title.as_deref(), *r#type, tag)
        }

        Some(Commands::Index { rebuild }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::index::execute(cli, &store, *rebuild)
        }

        Some(Commands::Search {
            query,
            r#type,
            tag,
            exclude_mocs,
        }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::search::execute(cli, &store, query, *r#type, tag.as_deref(), *exclude_mocs)
        }

        Some(Commands::Prime) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::prime::execute(cli, &store)
        }

        Some(Commands::Setup {
            list,
            tool,
            print,
            check,
            remove,
        }) => commands::setup::execute(cli, *list, tool.as_deref(), *print, *check, *remove),

        Some(Commands::Doctor { fix }) => {
            let store = match discover_or_open_store(cli, &root) {
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
            commands::doctor::execute(cli, &store, *fix)?;
            Ok(())
        }

        Some(Commands::Sync {
            validate,
            fix,
            commit,
            push,
        }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::sync::execute(cli, &store, *validate, *fix, *commit, *push)
        }

        Some(Commands::Context {
            note,
            tag,
            moc,
            query,
            max_chars,
            transitive,
            with_body,
            safety_banner,
        }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::context::execute(
                cli,
                &store,
                commands::context::ContextOptions {
                    note_ids: note,
                    tag: tag.as_deref(),
                    moc_id: moc.as_deref(),
                    query: query.as_deref(),
                    max_chars: *max_chars,
                    transitive: *transitive,
                    with_body: *with_body,
                    safety_banner: *safety_banner,
                },
            )
        }

        Some(Commands::Export {
            note,
            tag,
            moc,
            query,
            output,
            mode,
            with_attachments,
        }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            let export_mode = commands::export::ExportMode::parse(mode)?;
            commands::export::execute(
                cli,
                &store,
                commands::export::ExportOptions {
                    note_ids: note,
                    tag: tag.as_deref(),
                    moc_id: moc.as_deref(),
                    query: query.as_deref(),
                    output: output.as_deref(),
                    mode: export_mode,
                    with_attachments: *with_attachments,
                },
            )
        }

        Some(Commands::Link { command }) => {
            let store = discover_or_open_store(cli, &root)?;
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
                        .parse::<commands::link::Direction>()
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
                    commands::link::add::execute(cli, &store, from, to, *r#type)
                }
                LinkCommands::Remove { from, to, r#type } => {
                    commands::link::remove::execute(cli, &store, from, to, *r#type)
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
                        .parse::<commands::link::Direction>()
                        .map_err(QipuError::Other)?;
                    let opts = commands::link::TreeOptions {
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
                        .parse::<commands::link::Direction>()
                        .map_err(QipuError::Other)?;
                    let opts = commands::link::TreeOptions {
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
                    };
                    commands::link::path::execute(cli, &store, from, to, opts)
                }
            }
        }

        Some(Commands::Compact { command }) => commands::compact::execute(cli, command),

        Some(Commands::Dump {
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
        }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            let dir = direction
                .parse::<commands::link::Direction>()
                .map_err(QipuError::Other)?;

            commands::dump::execute(
                cli,
                &store,
                commands::dump::DumpOptions {
                    note_ids: note,
                    tag: tag.as_deref(),
                    moc_id: moc.as_deref(),
                    query: query.as_deref(),
                    direction: dir,
                    max_hops: *max_hops,
                    type_include: r#type.clone(),
                    typed_only: *typed_only,
                    inline_only: *inline_only,
                    include_attachments: !*no_attachments,
                    output: output.as_deref(),
                },
            )
        }

        Some(Commands::Load { pack_file }) => {
            let store = discover_or_open_store(cli, &root)?;
            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }
            commands::load::execute(cli, &store, pack_file)
        }
    }
}

fn discover_or_open_store(cli: &Cli, root: &PathBuf) -> Result<Store> {
    if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)
    } else {
        Store::discover(root)
    }
}
