use clap::Subcommand;

/// Value subcommands
#[derive(Subcommand, Debug)]
pub enum ValueCommands {
    /// Set the value of a note
    Set {
        /// Note ID or file path
        id_or_path: String,

        /// Value score (0-100)
        score: u8,
    },

    /// Show the value of a note
    Show {
        /// Note ID or file path
        id_or_path: String,
    },
}
