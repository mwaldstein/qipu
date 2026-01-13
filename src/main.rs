//! Qipu - Zettelkasten-inspired knowledge management CLI
//!
//! A command-line tool for capturing research, distilling insights,
//! and navigating knowledge via links, tags, and Maps of Content.

mod cli;
mod commands;
#[path = "lib/mod.rs"]
mod lib;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use chrono::DateTime;
use clap::Parser;

use cli::{Cli, Commands, LinkCommands, OutputFormat};
use lib::error::{ExitCode as QipuExitCode, QipuError};
use lib::store::Store;

fn main() -> ExitCode {
    let start = Instant::now();

    let argv_format_json = argv_requests_json();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // `--format` is a global flag, but clap may fail parsing before we can
            // inspect `Cli.format`. If the user requested JSON output, emit a
            // structured error envelope.
            if argv_format_json {
                let qipu_error = match err.kind() {
                    clap::error::ErrorKind::ValueValidation
                    | clap::error::ErrorKind::InvalidValue
                    | clap::error::ErrorKind::InvalidSubcommand
                    | clap::error::ErrorKind::UnknownArgument
                    | clap::error::ErrorKind::MissingRequiredArgument => {
                        QipuError::UsageError(err.to_string())
                    }
                    clap::error::ErrorKind::ArgumentConflict => {
                        // This includes duplicate `--format`.
                        QipuError::DuplicateFormat
                    }
                    _ => QipuError::Other(err.to_string()),
                };

                eprintln!("{}", qipu_error.to_json());
                return ExitCode::from(qipu_error.exit_code() as u8);
            }

            err.exit();
        }
    };

    if cli.verbose {
        eprintln!("parse_args: {:?}", start.elapsed());
    }

    let result = run(&cli, start);

    match result {
        Ok(()) => ExitCode::from(QipuExitCode::Success as u8),
        Err(e) => {
            let exit_code = e.exit_code();

            if cli.format == OutputFormat::Json {
                eprintln!("{}", e.to_json());
            } else if !cli.quiet {
                eprintln!("error: {}", e);
            }

            ExitCode::from(exit_code as u8)
        }
    }
}

fn argv_requests_json() -> bool {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--format" {
            if let Some(value) = args.next() {
                if value == "json" {
                    return true;
                }
            }
        }
    }
    false
}

fn run(cli: &Cli, start: Instant) -> Result<(), QipuError> {
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
            // Discover or require existing store
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::create::execute(cli, &store, &args.title, args.r#type, &args.tag, args.open)
        }

        Some(Commands::List { tag, r#type, since }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

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
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::show::execute(cli, &store, id_or_path, *links)
        }

        Some(Commands::Inbox { exclude_linked: _ }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            // Inbox is essentially list with type filter for fleeting/literature
            // For now, filter for fleeting and literature types
            let notes = store.list_notes()?;
            let inbox_notes: Vec<_> = notes
                .into_iter()
                .filter(|n| {
                    matches!(
                        n.note_type(),
                        lib::note::NoteType::Fleeting | lib::note::NoteType::Literature
                    )
                })
                .collect();

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
                                lib::note::NoteType::Fleeting => "F",
                                lib::note::NoteType::Literature => "L",
                                _ => "?",
                            };
                            println!("{} [{}] {}", note.id(), type_indicator, note.title());
                        }
                    }
                }
                OutputFormat::Records => {
                    // Header line per spec (specs/records-output.md)
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
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::capture::execute(cli, &store, title.as_deref(), *r#type, tag)
        }

        Some(Commands::Index { rebuild }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::index::execute(cli, &store, *rebuild)
        }

        Some(Commands::Search { query, r#type, tag }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::search::execute(cli, &store, query, *r#type, tag.as_deref())
        }

        Some(Commands::Prime) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::prime::execute(cli, &store)
        }

        Some(Commands::Doctor { fix }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                // Use open_unchecked for doctor to allow diagnosing corrupted stores
                Store::open_unchecked(&resolved)?
            } else {
                // Try discover first, fall back to unchecked if discovery fails
                match Store::discover(&root) {
                    Ok(store) => store,
                    Err(_) => {
                        // If discovery fails, try to find a .qipu directory and open unchecked
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
                }
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::doctor::execute(cli, &store, *fix)
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
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

            if cli.verbose {
                eprintln!("discover_store: {:?}", start.elapsed());
            }

            commands::context::execute(
                cli,
                &store,
                note,
                tag.as_deref(),
                moc.as_deref(),
                query.as_deref(),
                *max_chars,
                *transitive,
                *with_body,
                *safety_banner,
            )
        }

        Some(Commands::Link { command }) => {
            let store_path = cli.store.clone();
            let store = if let Some(path) = store_path {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                Store::open(&resolved)?
            } else {
                Store::discover(&root)?
            };

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
                } => {
                    let dir = direction
                        .parse::<commands::link::Direction>()
                        .map_err(|e| lib::error::QipuError::Other(e))?;
                    commands::link::execute_list(
                        cli,
                        &store,
                        id_or_path,
                        dir,
                        r#type.as_deref(),
                        *typed_only,
                        *inline_only,
                    )
                }
                LinkCommands::Add { from, to, r#type } => {
                    commands::link::execute_add(cli, &store, from, to, *r#type)
                }
                LinkCommands::Remove { from, to, r#type } => {
                    commands::link::execute_remove(cli, &store, from, to, *r#type)
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
                } => {
                    let dir = direction
                        .parse::<commands::link::Direction>()
                        .map_err(|e| lib::error::QipuError::Other(e))?;
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
                    };
                    commands::link::execute_tree(cli, &store, id_or_path, opts)
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
                } => {
                    let dir = direction
                        .parse::<commands::link::Direction>()
                        .map_err(|e| lib::error::QipuError::Other(e))?;
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
                    };
                    commands::link::execute_path(cli, &store, from, to, opts)
                }
            }
        }
    }
}
