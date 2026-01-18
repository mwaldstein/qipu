//! CLI argument parsing for qipu
//!
//! Uses clap for argument parsing per spec requirements.
//! Supports global flags: --root, --store, --format, --quiet, --verbose

pub mod args;
pub mod link;
pub mod output;
pub mod parse;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::lib::note::NoteType;
pub use args::CreateArgs;
pub use link::LinkCommands;
pub use output::OutputFormat;
use parse::parse_note_type;

/// Qipu - Zettelkasten-inspired knowledge management CLI
#[derive(Parser, Debug)]
#[command(name = "qipu")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Base directory for resolving the store
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    /// Explicit store root path
    #[arg(long, global = true)]
    pub store: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "human")]
    pub format: OutputFormat,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Report timing for major phases
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Disable compaction resolution (show raw compacted notes)
    #[arg(long, global = true)]
    pub no_resolve_compaction: bool,

    /// Include compacted note IDs in output
    #[arg(long, global = true)]
    pub with_compaction_ids: bool,

    /// Compaction traversal depth (requires --with-compaction-ids)
    #[arg(long, global = true)]
    pub compaction_depth: Option<u32>,

    /// Maximum compacted notes to include in output
    #[arg(long, global = true)]
    pub compaction_max_nodes: Option<usize>,

    /// Expand compacted notes to include full content (context command only)
    #[arg(long, global = true)]
    pub expand_compaction: bool,

    /// Target workspace name
    #[arg(long, global = true)]
    pub workspace: Option<String>,

    /// Disable semantic inversion for link listing/traversal
    #[arg(long, global = true)]
    pub no_semantic_inversion: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new qipu store
    Init {
        /// Use visible store directory (qipu/ instead of .qipu/)
        #[arg(long)]
        visible: bool,

        /// Stealth mode - add store to .gitignore
        #[arg(long)]
        stealth: bool,

        /// Protected branch workflow (store on separate branch)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Create a new note
    Create(CreateArgs),

    /// Alias for create
    New(CreateArgs),

    /// List notes
    List {
        /// Filter by tag
        #[arg(long, short)]
        tag: Option<String>,

        /// Filter by note type
        #[arg(long, short = 'T', value_parser = parse_note_type)]
        r#type: Option<NoteType>,

        /// Filter by creation date (ISO 8601)
        #[arg(long)]
        since: Option<String>,
    },

    /// Show a note
    Show {
        /// Note ID or file path
        id_or_path: String,

        /// Show links for the note (inline + typed links, both directions)
        #[arg(long)]
        links: bool,
    },

    /// List unprocessed notes (fleeting/literature)
    Inbox {
        /// Exclude notes already linked into a MOC
        #[arg(long)]
        exclude_linked: bool,
    },

    /// Create a new note from stdin
    Capture {
        /// Note title (auto-generated from content if not provided)
        #[arg(long, short = 't')]
        title: Option<String>,

        /// Note type
        #[arg(long, short = 'T', value_parser = parse_note_type)]
        r#type: Option<NoteType>,

        /// Tags (can be specified multiple times)
        #[arg(long, action = clap::ArgAction::Append)]
        tag: Vec<String>,

        /// The original source of the information
        #[arg(long)]
        source: Option<String>,

        /// Name of the human or agent who created the note
        #[arg(long)]
        author: Option<String>,

        /// Name of the LLM model used to generate the content
        #[arg(long)]
        generated_by: Option<String>,

        /// Hash or ID of the prompt used to generate the content
        #[arg(long)]
        prompt_hash: Option<String>,

        /// Flag indicating if a human has manually reviewed the content
        #[arg(long)]
        verified: Option<bool>,

        /// Note ID (for testing and advanced use cases)
        #[arg(long)]
        id: Option<String>,
    },

    /// Build or refresh derived indexes
    Index {
        /// Drop and regenerate indexes from scratch
        #[arg(long)]
        rebuild: bool,
    },

    /// Search notes by title and body
    Search {
        /// Search query
        query: String,

        /// Filter by note type
        #[arg(long, short = 'T', value_parser = parse_note_type)]
        r#type: Option<NoteType>,

        /// Filter by tag
        #[arg(long, short)]
        tag: Option<String>,

        /// Exclude MOCs from search results
        #[arg(long)]
        exclude_mocs: bool,
    },

    /// Toggle verification status of a note
    Verify {
        /// Note ID or file path
        id_or_path: String,

        /// Explicitly set verification status (true/false)
        #[arg(long)]
        status: Option<bool>,
    },

    /// Manage and traverse note links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },

    /// Output session-start primer for LLM agents
    Prime,

    /// Install qipu integration instructions for agent tools
    Setup {
        /// List available integrations
        #[arg(long)]
        list: bool,

        /// Tool/integration name (e.g., agents-md)
        tool: Option<String>,

        /// Print integration instructions to stdout
        #[arg(long)]
        print: bool,

        /// Check if integration is installed
        #[arg(long)]
        check: bool,

        /// Remove integration
        #[arg(long)]
        remove: bool,
    },

    /// Validate store invariants and optionally repair issues
    Doctor {
        /// Auto-repair issues where possible
        #[arg(long)]
        fix: bool,

        /// Check for near-duplicate notes using similarity
        #[arg(long)]
        duplicates: bool,

        /// Similarity threshold for duplicate detection (0.0 to 1.0)
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },

    /// Sync store: update indexes and optionally validate
    Sync {
        /// Run doctor validation after syncing
        #[arg(long)]
        validate: bool,

        /// Auto-repair issues if validation is enabled
        #[arg(long)]
        fix: bool,

        /// Commit changes to git if branch is configured
        #[arg(long)]
        commit: bool,

        /// Push changes to remote if branch is configured
        #[arg(long)]
        push: bool,
    },

    /// Build context bundle for LLM integration
    Context {
        /// Select notes by ID (can be repeated)
        #[arg(long, short = 'n', action = clap::ArgAction::Append)]
        note: Vec<String>,

        /// Select notes by tag
        #[arg(long)]
        tag: Option<String>,

        /// Select notes linked from a MOC
        #[arg(long, short = 'm')]
        moc: Option<String>,

        /// Select notes by search query
        #[arg(long)]
        query: Option<String>,

        /// Maximum output characters (exact budget)
        #[arg(long)]
        max_chars: Option<usize>,

        /// Maximum output tokens (approximate budget)
        #[arg(long)]
        max_tokens: Option<usize>,

        /// Tokenizer model to use (default: gpt-4o)
        #[arg(long, default_value = "gpt-4o")]
        model: String,

        /// Follow nested MOC links transitively
        #[arg(long)]
        transitive: bool,

        /// Include full note body content (records format)
        #[arg(long)]
        with_body: bool,

        /// Include safety banner for LLM prompt injection prevention
        #[arg(long)]
        safety_banner: bool,
    },

    /// Export notes to a single document
    Export {
        /// Select notes by ID (can be repeated)
        #[arg(long, short = 'n', action = clap::ArgAction::Append)]
        note: Vec<String>,

        /// Select notes by tag
        #[arg(long)]
        tag: Option<String>,

        /// Select notes linked from a MOC
        #[arg(long, short = 'm')]
        moc: Option<String>,

        /// Select notes by search query
        #[arg(long)]
        query: Option<String>,

        /// Output file path (default: stdout)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Export mode: bundle, outline, bibliography
        #[arg(long, default_value = "bundle")]
        mode: String,

        /// Copy referenced attachments to the output directory
        #[arg(long)]
        with_attachments: bool,

        /// Link handling: preserve, markdown, anchors
        #[arg(long, default_value = "preserve")]
        link_mode: String,
    },

    /// Manage note compaction (digest-first navigation)
    Compact {
        #[command(subcommand)]
        command: CompactCommands,
    },

    /// Manage and navigate isolated workspaces
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },

    /// Dump notes to a pack file
    Dump {
        /// Output pack file path
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Select notes by ID (can be repeated)
        #[arg(long, short = 'n', action = clap::ArgAction::Append)]
        note: Vec<String>,

        /// Select notes by tag
        #[arg(long)]
        tag: Option<String>,

        /// Select notes linked from a MOC
        #[arg(long, short = 'm')]
        moc: Option<String>,

        /// Select notes by search query
        #[arg(long)]
        query: Option<String>,

        /// Traversal direction
        #[arg(long, default_value = "both")]
        direction: String,

        /// Maximum traversal depth
        #[arg(long, default_value = "3")]
        max_hops: u32,

        /// Include only these link types (can be repeated)
        #[arg(long, short = 'T', action = clap::ArgAction::Append)]
        r#type: Vec<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,

        /// Exclude attachments from pack
        #[arg(long)]
        no_attachments: bool,

        /// Output file path (default: stdout)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },

    /// Load notes from a pack file
    Load {
        /// Pack file path
        pack_file: PathBuf,

        /// Conflict resolution strategy: skip, overwrite, merge-links
        #[arg(long, default_value = "skip")]
        strategy: String,
    },

    /// Merge note id1 into id2
    Merge {
        /// Source note ID (will be deleted)
        id1: String,
        /// Target note ID (will receive content and links)
        id2: String,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_help() {
        // Should not panic
        let result = Cli::try_parse_from(["qipu", "--help"]);
        assert!(result.is_err()); // --help exits
    }

    #[test]
    fn test_parse_cli_version() {
        // Should not panic
        let result = Cli::try_parse_from(["qipu", "--version"]);
        assert!(result.is_err()); // --version exits
    }

    #[test]
    fn test_parse_init() {
        let cli = Cli::try_parse_from(["qipu", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Init { .. })));
    }

    #[test]
    fn test_parse_create() {
        let cli = Cli::try_parse_from(["qipu", "create", "My Note"]).unwrap();
        if let Some(Commands::Create(args)) = cli.command {
            assert_eq!(args.title, "My Note");
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_parse_create_with_options() {
        let cli = Cli::try_parse_from([
            "qipu",
            "create",
            "My Note",
            "--type",
            "permanent",
            "--tag",
            "test",
            "--tag",
            "demo",
        ])
        .unwrap();
        if let Some(Commands::Create(args)) = cli.command {
            assert_eq!(args.title, "My Note");
            assert_eq!(args.r#type, Some(crate::lib::note::NoteType::Permanent));
            assert_eq!(args.tag, vec!["test", "demo"]);
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_parse_list() {
        let cli = Cli::try_parse_from(["qipu", "list"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::List { .. })));
    }

    #[test]
    fn test_parse_list_with_filters() {
        let cli =
            Cli::try_parse_from(["qipu", "list", "--tag", "test", "--type", "fleeting"]).unwrap();
        if let Some(Commands::List { tag, r#type, .. }) = cli.command {
            assert_eq!(tag, Some("test".to_string()));
            assert_eq!(r#type, Some(crate::lib::note::NoteType::Fleeting));
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_parse_format() {
        let cli = Cli::try_parse_from(["qipu", "--format", "json", "list"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Json);
    }
}
