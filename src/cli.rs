//! CLI argument parsing for qipu
//!
//! Uses clap for argument parsing per spec requirements.
//! Supports global flags: --root, --store, --format, --quiet, --verbose

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

pub use crate::lib::format::OutputFormat;
use crate::lib::note::{LinkType, NoteType};

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
    },

    /// Manage and traverse note links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },

    /// Output session-start primer for LLM agents
    Prime,

    /// Validate store invariants and optionally repair issues
    Doctor {
        /// Auto-repair issues where possible
        #[arg(long)]
        fix: bool,
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
}

/// Link subcommands
#[derive(Subcommand, Debug)]
pub enum LinkCommands {
    /// List links for a note
    List {
        /// Note ID or file path
        id_or_path: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Filter by link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T')]
        r#type: Option<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,
    },

    /// Add a typed link between notes
    Add {
        /// Source note ID
        from: String,

        /// Target note ID
        to: String,

        /// Link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T', value_parser = parse_link_type, default_value = "related")]
        r#type: LinkType,
    },

    /// Remove a typed link between notes
    Remove {
        /// Source note ID
        from: String,

        /// Target note ID
        to: String,

        /// Link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T', value_parser = parse_link_type, default_value = "related")]
        r#type: LinkType,
    },

    /// Show traversal tree from a note
    Tree {
        /// Note ID or file path
        id_or_path: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Maximum traversal depth
        #[arg(long, default_value = "3")]
        max_hops: u32,

        /// Include only these link types (can be repeated)
        #[arg(long, short = 'T', action = clap::ArgAction::Append)]
        r#type: Vec<String>,

        /// Exclude these link types (can be repeated)
        #[arg(long, action = clap::ArgAction::Append)]
        exclude_type: Vec<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,

        /// Maximum nodes to visit
        #[arg(long)]
        max_nodes: Option<usize>,

        /// Maximum edges to emit
        #[arg(long)]
        max_edges: Option<usize>,

        /// Maximum neighbors per node
        #[arg(long)]
        max_fanout: Option<usize>,
    },

    /// Find path between two notes
    Path {
        /// Starting note ID
        from: String,

        /// Target note ID
        to: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Maximum path length
        #[arg(long, default_value = "10")]
        max_hops: u32,

        /// Include only these link types (can be repeated)
        #[arg(long, short = 'T', action = clap::ArgAction::Append)]
        r#type: Vec<String>,

        /// Exclude these link types (can be repeated)
        #[arg(long, action = clap::ArgAction::Append)]
        exclude_type: Vec<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,
    },
}

#[derive(Args, Debug, Clone)]
pub struct CreateArgs {
    /// Note title
    pub title: String,

    /// Note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Tags (can be specified multiple times)
    #[arg(long, short, action = clap::ArgAction::Append)]
    pub tag: Vec<String>,

    /// Open in editor after creation
    #[arg(long, short)]
    pub open: bool,
}

/// Parse note type from string
fn parse_note_type(s: &str) -> Result<NoteType, String> {
    s.parse::<NoteType>().map_err(|e| e.to_string())
}

/// Parse link type from string
fn parse_link_type(s: &str) -> Result<LinkType, String> {
    s.parse::<LinkType>().map_err(|e| e.to_string())
}

// Implement ValueEnum for OutputFormat to work with clap
impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OutputFormat::Human,
            OutputFormat::Json,
            OutputFormat::Records,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            OutputFormat::Human => Some(clap::builder::PossibleValue::new("human")),
            OutputFormat::Json => Some(clap::builder::PossibleValue::new("json")),
            OutputFormat::Records => Some(clap::builder::PossibleValue::new("records")),
        }
    }
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
            assert_eq!(args.r#type, Some(NoteType::Permanent));
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
            assert_eq!(r#type, Some(NoteType::Fleeting));
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
