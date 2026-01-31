//! Compaction command implementations
//!
//! Per spec: specs/compaction.md

pub mod apply;
pub mod guide;
pub mod report;
pub mod show;
pub mod status;
pub mod suggest;
pub mod utils;

use crate::cli::{Cli, CompactCommands};
use qipu_core::error::Result;
use std::path::Path;

/// Execute compact subcommand
pub fn execute(cli: &Cli, root: &Path, command: &CompactCommands) -> Result<()> {
    match command {
        CompactCommands::Apply {
            digest_id,
            note,
            from_stdin,
            notes_file,
        } => apply::execute(cli, root, digest_id, note, *from_stdin, notes_file.as_ref()),
        CompactCommands::Show {
            digest_id,
            compaction_depth,
        } => show::execute(cli, root, digest_id, *compaction_depth),
        CompactCommands::Status { id } => status::execute(cli, root, id),
        CompactCommands::Report { digest_id } => report::execute(cli, root, digest_id),
        CompactCommands::Suggest => suggest::execute(cli, root),
        CompactCommands::Guide => guide::execute(cli, root),
    }
}
