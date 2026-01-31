//! Tags subcommands for tag management

use clap::Subcommand;

/// Tags subcommands
#[derive(Subcommand, Debug)]
pub enum TagsCommands {
    /// List all tags with their usage frequencies
    List {},
}
