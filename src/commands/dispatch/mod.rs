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

        Some(Commands::Init {
            visible,
            stealth,
            branch,
            no_index,
            index_strategy,
        }) => handlers::handle_init(
            cli,
            &root,
            *stealth,
            *visible,
            branch.clone(),
            *no_index,
            index_strategy.clone(),
        ),

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

        Some(Commands::Show {
            id_or_path,
            links,
            custom,
        }) => notes::handle_show(cli, &root, id_or_path, *links, *custom, start),

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
            quick,
            tag,
            r#type,
            recent,
            moc,
            status,
        }) => maintenance::handle_index(
            cli,
            &root,
            *rebuild,
            *rewrite_wiki_links,
            quick.clone(),
            tag.clone(),
            *r#type,
            *recent,
            moc.clone(),
            *status,
            start,
        ),

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

        Some(Commands::Value { command }) => handlers::handle_value(cli, &root, command, start),

        Some(Commands::Tags { command }) => handlers::handle_tags(cli, &root, command, start),

        Some(Commands::Custom { command }) => handlers::handle_custom(cli, &root, command, start),

        Some(Commands::Prime) => maintenance::handle_prime(cli, &root, start),

        Some(Commands::Onboard) => handlers::handle_onboard(cli),

        Some(Commands::Setup {
            list,
            tool,
            print,
            check,
            remove,
        }) => handlers::handle_setup(cli, *list, tool.as_deref(), *print, *check, *remove),

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
            walk,
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
            note,
            tag,
            moc,
            query,
            max_chars,
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
                walk.as_deref(),
                walk_direction.as_str(),
                *walk_max_hops,
                walk_type,
                walk_exclude_type,
                *walk_typed_only,
                *walk_inline_only,
                *walk_max_nodes,
                *walk_max_edges,
                *walk_max_fanout,
                *walk_min_value,
                *walk_ignore_value,
                note,
                tag.as_deref(),
                moc.as_deref(),
                query.as_deref(),
                *max_chars,
                *transitive,
                use_full_body,
                *safety_banner,
                *related,
                *backlinks,
                *min_value,
                custom_filter,
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

        Some(Commands::Compact { command }) => handlers::handle_compact(cli, command),

        Some(Commands::Workspace { command }) => handlers::handle_workspace(cli, command),

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

        Some(Commands::Edit { id_or_path, editor }) => {
            notes::handle_edit(cli, &root, id_or_path, editor.as_deref(), start)
        }

        Some(Commands::Update {
            id_or_path,
            title,
            r#type,
            tag,
            remove_tag,
            value,
        }) => notes::handle_update(
            cli,
            &root,
            id_or_path,
            title.as_deref(),
            *r#type,
            tag,
            remove_tag,
            *value,
            start,
        ),

        Some(Commands::Store { command }) => handlers::handle_store(cli, &root, command, start),
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
