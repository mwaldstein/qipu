use clap::Subcommand;

/// Custom metadata subcommands
#[derive(Subcommand, Debug)]
pub enum CustomCommands {
    /// Set a custom metadata field on a note
    Set {
        /// Note ID or file path
        id_or_path: String,

        /// Custom field key
        key: String,

        /// Custom field value (auto-detects type: number, boolean, string, JSON array/object)
        #[arg(allow_hyphen_values = true)]
        value: String,
    },

    /// Get a custom metadata field from a note
    Get {
        /// Note ID or file path
        id_or_path: String,

        /// Custom field key
        key: String,
    },

    /// Show all custom metadata for a note
    Show {
        /// Note ID or file path
        id_or_path: String,
    },

    /// Remove a custom metadata field from a note
    Unset {
        /// Note ID or file path
        id_or_path: String,

        /// Custom field key
        key: String,
    },
}
