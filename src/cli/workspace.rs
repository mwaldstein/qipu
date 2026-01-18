//! Workspace subcommands for managing isolated work areas

use clap::Subcommand;

/// Workspace subcommands
#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// List all available workspaces
    List,

    /// Create a new workspace
    New {
        /// Workspace name
        name: String,

        /// Mark as temporary
        #[arg(long)]
        temp: bool,

        /// Start with a fresh, empty store (default)
        #[arg(long)]
        empty: bool,

        /// Fork the entire primary store
        #[arg(long)]
        copy_primary: bool,

        /// Initialize with notes matching a tag
        #[arg(long)]
        from_tag: Option<String>,

        /// Initialize with a slice of the graph from a note
        #[arg(long)]
        from_note: Option<String>,

        /// Initialize with notes matching a search query
        #[arg(long)]
        from_query: Option<String>,
    },

    /// Delete a workspace
    Delete {
        /// Workspace name
        name: String,

        /// Force deletion of unmerged changes
        #[arg(long)]
        force: bool,
    },

    /// Merge contents of one workspace into another
    Merge {
        /// Source workspace name (or . for current/primary)
        source: String,

        /// Target workspace name (or . for current/primary)
        #[arg(default_value = ".")]
        target: String,

        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,

        /// Resolution strategy for ID collisions (skip, overwrite, merge-links)
        #[arg(long, default_value = "skip")]
        strategy: String,

        /// Delete the source workspace after successful merge
        #[arg(long)]
        delete_source: bool,
    },
}
