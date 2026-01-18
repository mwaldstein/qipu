//! Compaction subcommands for digest-first navigation

use clap::Subcommand;
use std::path::PathBuf;

/// Compact subcommands
#[derive(Subcommand, Debug)]
pub enum CompactCommands {
    /// Register compaction: digest compacts source notes
    Apply {
        /// Digest note ID that will compact the source notes
        digest_id: String,

        /// Source note IDs to be compacted (can be repeated)
        #[arg(long, short = 'n', action = clap::ArgAction::Append)]
        note: Vec<String>,

        /// Read note IDs from stdin (one per line)
        #[arg(long)]
        from_stdin: bool,

        /// Read note IDs from file (one per line)
        #[arg(long)]
        notes_file: Option<PathBuf>,
    },

    /// Show compaction set for a digest
    Show {
        /// Digest note ID
        digest_id: String,

        /// Show compaction tree depth (default: 1 for direct compaction only)
        #[arg(long, default_value = "1")]
        compaction_depth: u32,
    },

    /// Show compaction status for a note
    Status {
        /// Note ID
        id: String,
    },

    /// Report compaction quality metrics
    Report {
        /// Digest note ID
        digest_id: String,
    },

    /// Suggest compaction candidates
    Suggest,

    /// Print compaction guidance for LLM agents
    Guide,
}
