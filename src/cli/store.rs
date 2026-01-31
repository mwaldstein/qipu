//! Store subcommands for database operations

use clap::Subcommand;

/// Store subcommands
#[derive(Subcommand, Debug)]
pub enum StoreCommands {
    /// Show database and store statistics
    Stats {},
}
