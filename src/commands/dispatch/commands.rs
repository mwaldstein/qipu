//! Command implementations for all qipu commands

use crate::cli::Commands;
use crate::commands::dispatch::command::{Command, CommandContext};
use qipu_core::error::Result;

impl Command for Commands {
    fn execute(&self, ctx: &CommandContext) -> Result<()> {
        dispatch_command::execute(self, ctx)
    }
}

pub(super) mod dispatch_command {
    use super::*;

    use crate::cli::commands::{core::*, data::*, meta::*};
    use crate::cli::CreateArgs;
    use crate::cli::{
        compact::CompactCommands, custom::CustomCommands, link::LinkCommands,
        ontology::OntologyCommands, store::StoreCommands, tags::TagsCommands, value::ValueCommands,
        workspace::WorkspaceCommands,
    };
    use crate::commands::dispatch::handlers::{self, InitOptions, SetupOptions};
    use crate::commands::dispatch::io::{self, DumpParams, ExportParams, LoadParams};
    use crate::commands::dispatch::{link, maintenance, notes};

    pub(super) fn execute(cmd: &Commands, ctx: &CommandContext) -> Result<()> {
        match cmd {
            Commands::Init(args) => execute_init(ctx, args),
            Commands::Create(args) => execute_create(ctx, args),
            Commands::New(args) => execute_create(ctx, args),
            Commands::List(args) => execute_list(ctx, args),
            Commands::Show(args) => execute_show(ctx, args),
            Commands::Inbox(args) => execute_inbox(ctx, args),
            Commands::Capture(args) => execute_capture(ctx, args),
            Commands::Index(args) => execute_index(ctx, args),
            Commands::Search(args) => execute_search(ctx, args),
            Commands::Verify(args) => execute_verify(ctx, args),
            Commands::Value(subcmd) => execute_value(ctx, &subcmd.command),
            Commands::Tags(subcmd) => execute_tags(ctx, &subcmd.command),
            Commands::Custom(subcmd) => execute_custom(ctx, &subcmd.command),
            Commands::Prime(args) => execute_prime(ctx, args),
            Commands::Onboard => execute_onboard(ctx),
            Commands::Setup(args) => execute_setup(ctx, args),
            Commands::Doctor(args) => execute_doctor(ctx, args),
            Commands::Sync(args) => execute_sync(ctx, args),
            Commands::Context(args) => execute_context(ctx, args),
            Commands::Export(args) => execute_export(ctx, args),
            Commands::Dump(args) => execute_dump(ctx, args),
            Commands::Load(args) => execute_load(ctx, args),
            Commands::Merge(args) => execute_merge(ctx, args),
            Commands::Link(subcmd) => execute_link(ctx, &subcmd.command),
            Commands::Compact(subcmd) => execute_compact(ctx, &subcmd.command),
            Commands::Workspace(subcmd) => execute_workspace(ctx, &subcmd.command),
            Commands::Edit(args) => execute_edit(ctx, args),
            Commands::Update(args) => execute_update(ctx, args),
            Commands::Store(subcmd) => execute_store(ctx, &subcmd.command),
            Commands::Ontology(subcmd) => execute_ontology(ctx, &subcmd.command),
        }
    }

    fn execute_init(ctx: &CommandContext, args: &InitArgs) -> Result<()> {
        handlers::handle_init(
            ctx.cli,
            ctx.root,
            InitOptions {
                stealth: args.stealth,
                visible: args.visible,
                branch: args.branch.clone(),
                no_index: args.no_index,
                index_strategy: args.index_strategy.clone(),
            },
        )
    }

    fn execute_create(ctx: &CommandContext, args: &CreateArgs) -> Result<()> {
        notes::handle_create(ctx.cli, ctx.root, args, ctx.start)
    }

    fn execute_list(ctx: &CommandContext, args: &ListArgs) -> Result<()> {
        notes::handle_list(
            ctx.cli,
            ctx.root,
            args.tag.as_deref(),
            args.r#type.clone(),
            args.since.as_deref(),
            args.min_value,
            args.custom.as_deref(),
            args.show_custom,
            ctx.start,
        )
    }

    fn execute_show(ctx: &CommandContext, args: &ShowArgs) -> Result<()> {
        notes::handle_show(
            ctx.cli,
            ctx.root,
            &args.id_or_path,
            args.links,
            args.custom,
            ctx.start,
        )
    }

    fn execute_inbox(ctx: &CommandContext, args: &InboxArgs) -> Result<()> {
        notes::handle_inbox(ctx.cli, ctx.root, args.exclude_linked, ctx.start)
    }

    fn execute_capture(ctx: &CommandContext, args: &CaptureArgs) -> Result<()> {
        notes::handle_capture(
            ctx.cli,
            ctx.root,
            args.title.as_deref(),
            args.r#type.clone(),
            &args.tag,
            args.source.clone(),
            args.author.clone(),
            args.generated_by.clone(),
            args.prompt_hash.clone(),
            args.verified,
            args.id.as_deref(),
            ctx.start,
        )
    }

    fn execute_index(ctx: &CommandContext, args: &IndexArgs) -> Result<()> {
        maintenance::handle_index(
            ctx.cli,
            ctx.root,
            args.rebuild,
            args.resume,
            args.rewrite_wiki_links,
            args.quick,
            args.tag.clone(),
            args.r#type.clone(),
            args.recent,
            args.moc.clone(),
            args.status,
            ctx.start,
        )
    }

    fn execute_search(ctx: &CommandContext, args: &SearchArgs) -> Result<()> {
        notes::handle_search(
            ctx.cli,
            ctx.root,
            &args.query,
            args.r#type.clone(),
            args.tag.as_deref(),
            args.exclude_mocs,
            args.min_value,
            args.sort.as_deref(),
            ctx.start,
        )
    }

    fn execute_verify(ctx: &CommandContext, args: &VerifyArgs) -> Result<()> {
        notes::handle_verify(ctx.cli, ctx.root, &args.id_or_path, args.status, ctx.start)
    }

    fn execute_value(ctx: &CommandContext, command: &ValueCommands) -> Result<()> {
        handlers::handle_value(ctx.cli, ctx.root, command, ctx.start)
    }

    fn execute_tags(ctx: &CommandContext, command: &TagsCommands) -> Result<()> {
        handlers::handle_tags(ctx.cli, ctx.root, command, ctx.start)
    }

    fn execute_custom(ctx: &CommandContext, command: &CustomCommands) -> Result<()> {
        handlers::handle_custom(ctx.cli, ctx.root, command, ctx.start)
    }

    fn execute_prime(ctx: &CommandContext, args: &PrimeArgs) -> Result<()> {
        maintenance::handle_prime(ctx.cli, ctx.root, args.compact, args.minimal, ctx.start)
    }

    fn execute_onboard(ctx: &CommandContext) -> Result<()> {
        handlers::handle_onboard(ctx.cli)
    }

    fn execute_setup(ctx: &CommandContext, args: &SetupArgs) -> Result<()> {
        handlers::handle_setup(
            ctx.cli,
            SetupOptions {
                list: args.list,
                tool: args.tool.as_deref(),
                print: args.print,
                check: args.check,
                remove: args.remove,
            },
        )
    }

    fn execute_doctor(ctx: &CommandContext, args: &DoctorArgs) -> Result<()> {
        maintenance::handle_doctor(
            ctx.cli,
            ctx.root,
            args.fix,
            args.duplicates,
            args.threshold,
            args.check.as_deref(),
            ctx.start,
        )
    }

    fn execute_sync(ctx: &CommandContext, args: &SyncArgs) -> Result<()> {
        maintenance::handle_sync(
            ctx.cli,
            ctx.root,
            args.validate,
            args.fix,
            args.commit,
            args.push,
            ctx.start,
        )
    }

    fn execute_context(ctx: &CommandContext, args: &ContextArgs) -> Result<()> {
        use crate::commands::context::{execute as context_execute, ContextOptions};
        let store = ctx.discover_or_open_store()?;
        let use_full_body = !args.summary_only || args.with_body;

        let options = ContextOptions {
            walk_id: args.walk.as_deref(),
            walk_direction: args.walk_direction.as_str(),
            walk_max_hops: args.walk_max_hops,
            walk_type: &args.walk_type,
            walk_exclude_type: &args.walk_exclude_type,
            walk_typed_only: args.walk_typed_only,
            walk_inline_only: args.walk_inline_only,
            walk_max_nodes: args.walk_max_nodes,
            walk_max_edges: args.walk_max_edges,
            walk_max_fanout: args.walk_max_fanout,
            walk_min_value: args.walk_min_value,
            walk_ignore_value: args.walk_ignore_value,
            note_ids: &args.note,
            tag: args.tag.as_deref(),
            moc_id: args.moc.as_deref(),
            query: args.query.as_deref(),
            max_chars: args.max_chars,
            transitive: args.transitive,
            with_body: use_full_body,
            safety_banner: args.safety_banner,
            related_threshold: if args.related > 0.0 {
                Some(args.related)
            } else {
                None
            },
            backlinks: args.backlinks,
            min_value: args.min_value,
            custom_filter: &args.custom_filter,
            include_custom: args.custom,
            include_ontology: args.include_ontology,
        };

        context_execute(ctx.cli, &store, options)
    }

    fn execute_export(ctx: &CommandContext, args: &ExportArgs) -> Result<()> {
        io::handle_export(ExportParams {
            cli: ctx.cli,
            root: ctx.root,
            note_ids: &args.note,
            tag: args.tag.as_deref(),
            moc_id: args.moc.as_deref(),
            query: args.query.as_deref(),
            output: args.output.as_ref(),
            mode: &args.mode,
            with_attachments: args.with_attachments,
            link_mode: &args.link_mode,
            bib_format: &args.bib_format,
            max_hops: args.max_hops,
            pdf: args.pdf,
            start: ctx.start,
        })
    }

    fn execute_dump(ctx: &CommandContext, args: &DumpArgs) -> Result<()> {
        io::handle_dump(DumpParams {
            cli: ctx.cli,
            root: ctx.root,
            file: args.file.as_ref(),
            note_ids: &args.note,
            tag: args.tag.as_deref(),
            moc_id: args.moc.as_deref(),
            query: args.query.as_deref(),
            direction: &args.direction,
            max_hops: args.max_hops,
            type_include: args.r#type.clone(),
            typed_only: args.typed_only,
            inline_only: args.inline_only,
            no_attachments: args.no_attachments,
            output: args.output.as_ref(),
            start: ctx.start,
        })
    }

    fn execute_load(ctx: &CommandContext, args: &LoadArgs) -> Result<()> {
        io::handle_load(LoadParams {
            cli: ctx.cli,
            root: ctx.root,
            pack_file: &args.pack_file,
            strategy: &args.strategy,
            apply_config: args.apply_config,
            start: ctx.start,
        })
    }

    fn execute_merge(ctx: &CommandContext, args: &MergeArgs) -> Result<()> {
        notes::handle_merge(
            ctx.cli,
            ctx.root,
            &args.id1,
            &args.id2,
            args.dry_run,
            ctx.start,
        )
    }

    fn execute_link(ctx: &CommandContext, command: &LinkCommands) -> Result<()> {
        link::handle_link(ctx.cli, ctx.root, command, ctx.start)
    }

    fn execute_compact(ctx: &CommandContext, command: &CompactCommands) -> Result<()> {
        handlers::handle_compact(ctx.cli, command)
    }

    fn execute_workspace(ctx: &CommandContext, command: &WorkspaceCommands) -> Result<()> {
        handlers::handle_workspace(ctx.cli, command)
    }

    fn execute_edit(ctx: &CommandContext, args: &EditArgs) -> Result<()> {
        notes::handle_edit(
            ctx.cli,
            ctx.root,
            &args.id_or_path,
            args.editor.as_deref(),
            ctx.start,
        )
    }

    fn execute_update(ctx: &CommandContext, args: &UpdateArgs) -> Result<()> {
        notes::handle_update(
            ctx.cli,
            ctx.root,
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
            ctx.start,
        )
    }

    fn execute_store(ctx: &CommandContext, command: &StoreCommands) -> Result<()> {
        handlers::handle_store(ctx.cli, ctx.root, command, ctx.start)
    }

    fn execute_ontology(ctx: &CommandContext, command: &OntologyCommands) -> Result<()> {
        handlers::handle_ontology(ctx.cli, ctx.root, command, ctx.start)
    }
}
