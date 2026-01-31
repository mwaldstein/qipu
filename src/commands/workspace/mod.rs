pub mod delete;
pub mod list;
pub mod merge;
pub mod new;

use crate::cli::{Cli, WorkspaceCommands};
use qipu_core::error::Result;
use std::path::Path;

pub fn execute(cli: &Cli, root: &Path, command: &WorkspaceCommands) -> Result<()> {
    match command {
        WorkspaceCommands::List => list::execute(cli, root),
        WorkspaceCommands::New {
            name,
            temp,
            empty,
            copy_primary,
            from_tag,
            from_note,
            from_query,
        } => new::execute(
            cli,
            root,
            name,
            *temp,
            *empty,
            *copy_primary,
            from_tag.as_deref(),
            from_note.as_deref(),
            from_query.as_deref(),
        ),
        WorkspaceCommands::Delete { name, force } => delete::execute(cli, root, name, *force),
        WorkspaceCommands::Merge {
            source,
            target,
            dry_run,
            strategy,
            delete_source,
        } => merge::execute(
            cli,
            root,
            source,
            target,
            *dry_run,
            strategy,
            *delete_source,
        ),
    }
}
