//! Command dispatch logic for qipu

#![allow(clippy::ptr_arg)]

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::{Cli, Commands, TagsCommands};
use crate::lib::error::{QipuError, Result};
use crate::lib::store::Store;
use tracing::debug;

mod io;
mod link;
mod maintenance;
mod notes;

pub fn run(cli: &Cli, start: Instant) -> Result<()> {
    // Determine the root directory
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    debug!(elapsed = ?start.elapsed(), "resolve_root");

    // Handle commands
    match &cli.command {
        None => handle_no_command(),

        Some(Commands::Init {
            visible,
            stealth,
            branch,
        }) => handle_init(cli, &root, *stealth, *visible, branch.clone()),

        Some(Commands::Create(args)) | Some(Commands::New(args)) => {
            notes::handle_create(cli, &root, args, start)
        }

        Some(Commands::List {
            tag,
            r#type,
            since,
            min_value,
            custom,
        }) => notes::handle_list(
            cli,
            &root,
            tag.as_deref(),
            *r#type,
            since.as_deref(),
            *min_value,
            custom.as_deref(),
            start,
        ),

        Some(Commands::Show { id_or_path, links }) => {
            notes::handle_show(cli, &root, id_or_path, *links, start)
        }

        Some(Commands::Inbox { exclude_linked }) => {
            notes::handle_inbox(cli, &root, *exclude_linked, start)
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
        }) => notes::handle_capture(
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

        Some(Commands::Index {
            rebuild,
            rewrite_wiki_links,
        }) => maintenance::handle_index(cli, &root, *rebuild, *rewrite_wiki_links, start),

        Some(Commands::Search {
            query,
            r#type,
            tag,
            exclude_mocs,
            min_value,
            sort,
        }) => notes::handle_search(
            cli,
            &root,
            query,
            *r#type,
            tag.as_deref(),
            *exclude_mocs,
            *min_value,
            sort.as_deref(),
            start,
        ),

        Some(Commands::Verify { id_or_path, status }) => {
            notes::handle_verify(cli, &root, id_or_path, *status, start)
        }

        Some(Commands::Value { command }) => handle_value(cli, &root, command, start),

        Some(Commands::Tags { command }) => handle_tags(cli, &root, command, start),

        Some(Commands::Custom { command }) => handle_custom(cli, &root, command, start),

        Some(Commands::Prime) => maintenance::handle_prime(cli, &root, start),

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
        }) => maintenance::handle_doctor(cli, &root, *fix, *duplicates, *threshold, start),

        Some(Commands::Sync {
            validate,
            fix,
            commit,
            push,
        }) => maintenance::handle_sync(cli, &root, *validate, *fix, *commit, *push, start),

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
            summary_only,
            safety_banner,
            related,
            backlinks,
            min_value,
            custom_filter,
            custom,
        }) => {
            // Default to full body unless --summary-only is specified
            // --with-body is kept for backward compatibility but is now the default
            let use_full_body = !summary_only || *with_body;
            notes::handle_context(
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
                use_full_body,
                *safety_banner,
                *related,
                *backlinks,
                *min_value,
                custom_filter.as_deref(),
                *custom,
                start,
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
            link_mode,
            bib_format,
            max_hops,
            pdf,
        }) => io::handle_export(
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
            bib_format,
            *max_hops,
            *pdf,
            start,
        ),

        Some(Commands::Link { command }) => link::handle_link(cli, &root, command, start),

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
        }) => io::handle_dump(
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
        }) => io::handle_load(cli, &root, pack_file, strategy, start),

        Some(Commands::Merge { id1, id2, dry_run }) => {
            notes::handle_merge(cli, &root, id1, id2, *dry_run, start)
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
// Local Command Handlers (not moved to submodules)
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
    crate::commands::init::execute(cli, root, stealth, visible, branch)
}

fn handle_setup(
    cli: &Cli,
    list: bool,
    tool: Option<&str>,
    print: bool,
    check: bool,
    remove: bool,
) -> Result<()> {
    crate::commands::setup::execute(cli, list, tool, print, check, remove)
}

fn handle_compact(cli: &Cli, command: &crate::cli::CompactCommands) -> Result<()> {
    crate::commands::compact::execute(cli, command)
}

fn handle_workspace(cli: &Cli, command: &crate::cli::WorkspaceCommands) -> Result<()> {
    crate::commands::workspace::execute(cli, command)
}

fn handle_value(
    cli: &Cli,
    root: &PathBuf,
    command: &crate::cli::ValueCommands,
    start: Instant,
) -> Result<()> {
    use crate::cli::ValueCommands;
    use std::fs;
    use std::path::Path;

    let store = discover_or_open_store(cli, root)?;

    match command {
        ValueCommands::Set { id_or_path, score } => {
            if *score > 100 {
                return Err(QipuError::UsageError(
                    "Value score must be between 0 and 100".to_string(),
                ));
            }

            let mut note = if Path::new(id_or_path).exists() {
                let content = fs::read_to_string(id_or_path)?;
                crate::lib::note::Note::parse(&content, Some(id_or_path.into()))?
            } else {
                store.get_note(id_or_path)?
            };

            let note_id = note.id().to_string();

            note.frontmatter.value = Some(*score);

            store.save_note(&mut note)?;

            println!("{}: {}", note_id, score);

            debug!(elapsed = ?start.elapsed(), "value_set");
            Ok(())
        }

        ValueCommands::Show { id_or_path } => {
            let note = if Path::new(id_or_path).exists() {
                let content = fs::read_to_string(id_or_path)?;
                crate::lib::note::Note::parse(&content, Some(id_or_path.into()))?
            } else {
                store.get_note(id_or_path)?
            };

            let note_id = note.id().to_string();
            let value = note.frontmatter.value.unwrap_or(50);

            if note.frontmatter.value.is_some() {
                println!("{}: {}", note_id, value);
            } else {
                println!("{}: {} (default)", note_id, value);
            }

            debug!(elapsed = ?start.elapsed(), "value_show");
            Ok(())
        }
    }
}

fn handle_tags(cli: &Cli, root: &PathBuf, command: &TagsCommands, start: Instant) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;

    match command {
        TagsCommands::List {} => {
            use crate::cli::OutputFormat;

            let frequencies = store.get_tag_frequencies()?;

            match cli.format {
                OutputFormat::Json => {
                    let output: Vec<_> = frequencies
                        .iter()
                        .map(|(tag, count)| {
                            serde_json::json!({
                                "tag": tag,
                                "count": count
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                OutputFormat::Human => {
                    if frequencies.is_empty() {
                        if !cli.quiet {
                            println!("No tags found");
                        }
                    } else {
                        for (tag, count) in &frequencies {
                            println!("{}: {}", tag, count);
                        }
                    }
                }
                OutputFormat::Records => {
                    if frequencies.is_empty() {
                        if !cli.quiet {
                            println!("No tags found");
                        }
                    } else {
                        for (tag, count) in &frequencies {
                            println!("T tag=\"{}\" count={}", tag, count);
                        }
                    }
                }
            }

            debug!(elapsed = ?start.elapsed(), "tags_list");
            Ok(())
        }
    }
}

fn handle_custom(
    cli: &Cli,
    root: &PathBuf,
    command: &crate::cli::CustomCommands,
    start: Instant,
) -> Result<()> {
    use crate::cli::CustomCommands;

    let store = discover_or_open_store(cli, root)?;

    match command {
        CustomCommands::Set {
            id_or_path,
            key,
            value,
        } => {
            crate::commands::custom::set_custom_field(&store, id_or_path, key, value, cli.quiet)?;
            debug!(elapsed = ?start.elapsed(), "custom_set");
            Ok(())
        }

        CustomCommands::Get { id_or_path, key } => {
            crate::commands::custom::get_custom_field(&store, id_or_path, key)?;
            debug!(elapsed = ?start.elapsed(), "custom_get");
            Ok(())
        }

        CustomCommands::Show { id_or_path } => {
            crate::commands::custom::show_custom_fields(&store, id_or_path)?;
            debug!(elapsed = ?start.elapsed(), "custom_show");
            Ok(())
        }

        CustomCommands::Unset { id_or_path, key } => {
            crate::commands::custom::unset_custom_field(&store, id_or_path, key, cli.quiet)?;
            debug!(elapsed = ?start.elapsed(), "custom_unset");
            Ok(())
        }
    }
}
