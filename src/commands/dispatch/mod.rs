//! Command dispatch logic for qipu

#![allow(clippy::ptr_arg)]

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::{Cli, Commands};
use crate::lib::error::Result;
use crate::lib::store::Store;
use tracing::debug;

mod handlers;
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
        None => handlers::handle_no_command(),

        Some(Commands::Init(args)) => handlers::handle_init(
            cli,
            &root,
            args.stealth,
            args.visible,
            args.branch.clone(),
            args.no_index,
            args.index_strategy.clone(),
        ),

        Some(Commands::Create(args)) | Some(Commands::New(args)) => {
            notes::handle_create(cli, &root, args, start)
        }

        Some(Commands::List(args)) => notes::handle_list(
            cli,
            &root,
            args.tag.as_deref(),
            args.r#type.clone(),
            args.since.as_deref(),
            args.min_value,
            args.custom.as_deref(),
            args.show_custom,
            start,
        ),

        Some(Commands::Show(args)) => {
            notes::handle_show(cli, &root, &args.id_or_path, args.links, args.custom, start)
        }

        Some(Commands::Inbox(args)) => notes::handle_inbox(cli, &root, args.exclude_linked, start),

        Some(Commands::Capture(args)) => notes::handle_capture(
            cli,
            &root,
            args.title.as_deref(),
            args.r#type.clone(),
            &args.tag,
            args.source.clone(),
            args.author.clone(),
            args.generated_by.clone(),
            args.prompt_hash.clone(),
            args.verified,
            args.id.as_deref(),
            start,
        ),

        Some(Commands::Index(args)) => maintenance::handle_index(
            cli,
            &root,
            args.rebuild,
            args.resume,
            args.rewrite_wiki_links,
            args.quick,
            args.tag.clone(),
            args.r#type.clone(),
            args.recent,
            args.moc.clone(),
            args.status,
            start,
        ),

        Some(Commands::Search(args)) => notes::handle_search(
            cli,
            &root,
            &args.query,
            args.r#type.clone(),
            args.tag.as_deref(),
            args.exclude_mocs,
            args.min_value,
            args.sort.as_deref(),
            start,
        ),

        Some(Commands::Verify(args)) => {
            notes::handle_verify(cli, &root, &args.id_or_path, args.status, start)
        }

        Some(Commands::Value(args)) => handlers::handle_value(cli, &root, &args.command, start),

        Some(Commands::Tags(args)) => handlers::handle_tags(cli, &root, &args.command, start),

        Some(Commands::Custom(args)) => handlers::handle_custom(cli, &root, &args.command, start),

        Some(Commands::Prime(args)) => {
            maintenance::handle_prime(cli, &root, args.compact, args.minimal, start)
        }

        Some(Commands::Onboard) => handlers::handle_onboard(cli),

        Some(Commands::Setup(args)) => handlers::handle_setup(
            cli,
            args.list,
            args.tool.as_deref(),
            args.print,
            args.check,
            args.remove,
        ),

        Some(Commands::Doctor(args)) => maintenance::handle_doctor(
            cli,
            &root,
            args.fix,
            args.duplicates,
            args.threshold,
            args.check.as_deref(),
            start,
        ),

        Some(Commands::Sync(args)) => maintenance::handle_sync(
            cli,
            &root,
            args.validate,
            args.fix,
            args.commit,
            args.push,
            start,
        ),

        Some(Commands::Context(args)) => {
            // Default to full body unless --summary-only is specified
            // --with-body is kept for backward compatibility but is now the default
            let use_full_body = !args.summary_only || args.with_body;
            notes::handle_context(
                cli,
                &root,
                args.walk.as_deref(),
                args.walk_direction.as_str(),
                args.walk_max_hops,
                &args.walk_type,
                &args.walk_exclude_type,
                args.walk_typed_only,
                args.walk_inline_only,
                args.walk_max_nodes,
                args.walk_max_edges,
                args.walk_max_fanout,
                args.walk_min_value,
                args.walk_ignore_value,
                &args.note,
                args.tag.as_deref(),
                args.moc.as_deref(),
                args.query.as_deref(),
                args.max_chars,
                args.transitive,
                use_full_body,
                args.safety_banner,
                args.related,
                args.backlinks,
                args.min_value,
                &args.custom_filter,
                args.custom,
                args.include_ontology,
                start,
            )
        }

        Some(Commands::Export(args)) => io::handle_export(
            cli,
            &root,
            &args.note,
            args.tag.as_deref(),
            args.moc.as_deref(),
            args.query.as_deref(),
            args.output.as_ref(),
            &args.mode,
            args.with_attachments,
            &args.link_mode,
            &args.bib_format,
            args.max_hops,
            args.pdf,
            start,
        ),

        Some(Commands::Dump(args)) => io::handle_dump(
            cli,
            &root,
            args.file.as_ref(),
            &args.note,
            args.tag.as_deref(),
            args.moc.as_deref(),
            args.query.as_deref(),
            &args.direction,
            args.max_hops,
            args.r#type.clone(),
            args.typed_only,
            args.inline_only,
            args.no_attachments,
            args.output.as_ref(),
            start,
        ),

        Some(Commands::Load(args)) => io::handle_load(
            cli,
            &root,
            &args.pack_file,
            &args.strategy,
            args.apply_config,
            start,
        ),

        Some(Commands::Merge(args)) => {
            notes::handle_merge(cli, &root, &args.id1, &args.id2, args.dry_run, start)
        }

        Some(Commands::Link(args)) => link::handle_link(cli, &root, &args.command, start),

        Some(Commands::Compact(args)) => handlers::handle_compact(cli, &args.command),

        Some(Commands::Workspace(args)) => handlers::handle_workspace(cli, &args.command),

        Some(Commands::Edit(args)) => {
            notes::handle_edit(cli, &root, &args.id_or_path, args.editor.as_deref(), start)
        }

        Some(Commands::Update(args)) => notes::handle_update(
            cli,
            &root,
            &args.id_or_path,
            args.title.as_deref(),
            args.r#type.clone(),
            &args.tag,
            &args.remove_tag,
            args.value,
            args.source.as_deref(),
            args.author.as_deref(),
            args.generated_by.as_deref(),
            args.prompt_hash.as_deref(),
            args.verified,
            start,
        ),

        Some(Commands::Store(args)) => handlers::handle_store(cli, &root, &args.command, start),

        Some(Commands::Ontology(args)) => {
            handlers::handle_ontology(cli, &root, &args.command, start)
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
