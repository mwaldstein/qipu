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
    use crate::cli::link::LinkCommands;
    use crate::cli::CreateArgs;
    use crate::commands::dispatch::handlers::{self, InitOptions, SetupOptions};
    use crate::commands::dispatch::io;
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
            Commands::Value(subcmd) => {
                handlers::execute_value(ctx.cli, ctx.root, &subcmd.command, ctx.start)
            }
            Commands::Tags(subcmd) => {
                handlers::execute_tags(ctx.cli, ctx.root, &subcmd.command, ctx.start)
            }
            Commands::Custom(subcmd) => {
                handlers::execute_custom(ctx.cli, ctx.root, &subcmd.command, ctx.start)
            }
            Commands::Prime(args) => execute_prime(ctx, args),
            Commands::Onboard => handlers::execute_onboard(ctx.cli),
            Commands::Setup(args) => execute_setup(ctx, args),
            Commands::Doctor(args) => execute_doctor(ctx, args),
            Commands::Sync(args) => execute_sync(ctx, args),
            Commands::Context(args) => execute_context(ctx, args),
            Commands::Export(args) => execute_export(ctx, args),
            Commands::Dump(args) => execute_dump(ctx, args),
            Commands::Load(args) => execute_load(ctx, args),
            Commands::Merge(args) => execute_merge(ctx, args),
            Commands::Link(subcmd) => execute_link(ctx, &subcmd.command),
            Commands::Compact(subcmd) => handlers::execute_compact(ctx.cli, &subcmd.command),
            Commands::Workspace(subcmd) => handlers::execute_workspace(ctx.cli, &subcmd.command),
            Commands::Edit(args) => execute_edit(ctx, args),
            Commands::Update(args) => execute_update(ctx, args),
            Commands::Store(subcmd) => {
                handlers::execute_store(ctx.cli, ctx.root, &subcmd.command, ctx.start)
            }
            Commands::Ontology(subcmd) => {
                handlers::execute_ontology_dispatch(ctx.cli, ctx.root, &subcmd.command, ctx.start)
            }
        }
    }

    fn execute_init(ctx: &CommandContext, args: &InitArgs) -> Result<()> {
        handlers::handle_init(
            ctx.cli,
            ctx.root,
            InitOptions {
                stealth: args.stealth,
                visible: args.visible,
                branch: args.branch.as_deref(),
                no_index: args.no_index,
                index_strategy: args.index_strategy.as_deref(),
            },
        )
    }

    fn execute_create(ctx: &CommandContext, args: &CreateArgs) -> Result<()> {
        notes::handle_create(ctx.cli, ctx.root, args, ctx.start)
    }

    fn execute_list(ctx: &CommandContext, args: &ListArgs) -> Result<()> {
        use crate::commands::dispatch::notes::ListOptions;
        notes::handle_list(
            ctx.cli,
            ctx.root,
            ListOptions {
                tag: args.tag.as_deref(),
                note_type: args.r#type.clone(),
                since: args.since.as_deref(),
                min_value: args.min_value,
                custom: args.custom.as_deref(),
                show_custom: args.show_custom,
                start: ctx.start,
            },
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
        notes::execute_capture_from_args(ctx.cli, ctx.root, args, ctx.start)
    }

    fn execute_index(ctx: &CommandContext, args: &IndexArgs) -> Result<()> {
        maintenance::handle_index(
            ctx.cli,
            ctx.root,
            args.rebuild,
            args.resume,
            args.rewrite_wiki_links,
            args.quick,
            args.tag.as_deref(),
            args.r#type.clone(),
            args.recent,
            args.moc.as_deref(),
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

    fn execute_prime(ctx: &CommandContext, args: &PrimeArgs) -> Result<()> {
        maintenance::handle_prime(
            ctx.cli,
            ctx.root,
            args.compact,
            args.minimal,
            args.full,
            args.mcp,
            args.use_prime_md,
            ctx.start,
        )
    }

    fn execute_setup(ctx: &CommandContext, args: &SetupArgs) -> Result<()> {
        handlers::execute_setup(
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
        crate::commands::context::execute_with_args(ctx.cli, ctx.root, args, ctx.start)
    }

    fn execute_export(ctx: &CommandContext, args: &ExportArgs) -> Result<()> {
        io::execute_export(
            ctx.cli,
            ctx.root,
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
            ctx.start,
        )
    }

    fn execute_dump(ctx: &CommandContext, args: &DumpArgs) -> Result<()> {
        io::execute_dump(
            ctx.cli,
            ctx.root,
            args.file.as_ref(),
            &args.note,
            args.tag.as_deref(),
            args.moc.as_deref(),
            args.query.as_deref(),
            &args.direction,
            args.max_hops,
            &args.r#type,
            args.typed_only,
            args.inline_only,
            args.no_attachments,
            args.output.as_ref(),
            ctx.start,
        )
    }

    fn execute_load(ctx: &CommandContext, args: &LoadArgs) -> Result<()> {
        io::execute_load_dispatch(
            ctx.cli,
            ctx.root,
            &args.pack_file,
            &args.strategy,
            args.apply_config,
            ctx.start,
        )
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
        notes::execute_update_from_args(ctx.cli, ctx.root, args, ctx.start)
    }
}
