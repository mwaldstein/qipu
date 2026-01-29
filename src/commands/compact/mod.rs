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

/// Execute compact subcommand
pub fn execute(cli: &Cli, command: &CompactCommands) -> Result<()> {
    match command {
        CompactCommands::Apply {
            digest_id,
            note,
            from_stdin,
            notes_file,
        } => apply::execute(cli, digest_id, note, *from_stdin, notes_file.as_ref()),
        CompactCommands::Show {
            digest_id,
            compaction_depth,
        } => show::execute(cli, digest_id, *compaction_depth),
        CompactCommands::Status { id } => status::execute(cli, id),
        CompactCommands::Report { digest_id } => report::execute(cli, digest_id),
        CompactCommands::Suggest => suggest::execute(cli),
        CompactCommands::Guide => guide::execute(cli),
    }
}
