//! Core command argument structures

use clap::Args;

use crate::cli::parse::parse_note_type;
use qipu_core::note::NoteType;

/// Arguments for the init command.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Use visible store directory (qipu/ instead of .qipu/)
    #[arg(long)]
    pub visible: bool,

    /// Stealth mode - add store to .gitignore
    #[arg(long)]
    pub stealth: bool,

    /// Protected branch workflow (store on separate branch)
    #[arg(long)]
    pub branch: Option<String>,

    /// Skip automatic indexing
    #[arg(long)]
    pub no_index: bool,

    /// Override auto-indexing strategy (adaptive, full, incremental, quick)
    #[arg(long)]
    pub index_strategy: Option<String>,

    /// Write qipu section to AGENTS.md
    #[arg(long)]
    pub agents_md: bool,
}

/// Arguments for the list command.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by tag
    #[arg(long, short)]
    pub tag: Option<String>,

    /// Filter by note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Filter by creation date (ISO 8601)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter by minimum value (0-100, default: 50)
    #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
    pub min_value: Option<u8>,

    /// Filter by custom metadata (format: key=value)
    #[arg(long)]
    pub custom: Option<String>,

    /// Include custom metadata in output (opt-in)
    #[arg(long)]
    pub show_custom: bool,
}

/// Arguments for the show command.
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Note ID or file path
    pub id_or_path: String,

    /// Show links for the note (inline + typed links, both directions)
    #[arg(long)]
    pub links: bool,

    /// Include custom metadata in output (opt-in)
    #[arg(long)]
    pub custom: bool,
}

/// Arguments for the inbox command.
#[derive(Args, Debug)]
pub struct InboxArgs {
    /// Exclude notes already linked into a MOC
    #[arg(long)]
    pub exclude_linked: bool,
}

/// Arguments for the capture command.
#[derive(Args, Debug)]
pub struct CaptureArgs {
    /// Note title (auto-generated from content if not provided)
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// Note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Tags (can be specified multiple times)
    #[arg(long, action = clap::ArgAction::Append)]
    pub tag: Vec<String>,

    /// The original source of the information
    #[arg(long)]
    pub source: Option<String>,

    /// Name of the human or agent who created the note
    #[arg(long)]
    pub author: Option<String>,

    /// Name of the LLM model used to generate the content
    #[arg(long)]
    pub generated_by: Option<String>,

    /// Hash or ID of the prompt used to generate the content
    #[arg(long)]
    pub prompt_hash: Option<String>,

    /// Flag indicating if a human has manually reviewed the content
    #[arg(long)]
    pub verified: Option<bool>,

    /// Note ID (for testing and advanced use cases)
    #[arg(long)]
    pub id: Option<String>,
}

/// Arguments for the index command.
#[derive(Args, Debug)]
pub struct IndexArgs {
    /// Drop and regenerate indexes from scratch
    #[arg(long)]
    pub rebuild: bool,

    /// Force full re-index (alias for --rebuild)
    #[arg(long)]
    pub force: bool,

    /// Resume from last checkpoint (for interrupted rebuild)
    #[arg(long)]
    pub resume: bool,

    /// Basic index only (metadata, no full-text)
    #[arg(long)]
    pub basic: bool,

    /// Full-text index (includes body content)
    #[arg(long)]
    pub full: bool,

    /// Rewrite wiki-links to markdown links (optional; opt-in)
    #[arg(long)]
    pub rewrite_wiki_links: bool,

    /// Index only MOCs + recent 100 notes
    #[arg(long)]
    pub quick: bool,

    /// Index only notes with specified tag
    #[arg(long, short = 't')]
    pub tag: Option<String>,

    /// Index only notes of specified type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Index N most recently updated notes
    #[arg(long)]
    pub recent: Option<usize>,

    /// Index MOC and its linked notes
    #[arg(long)]
    pub moc: Option<String>,

    /// Index only notes modified since timestamp
    #[arg(long)]
    pub modified_since: Option<String>,

    /// Set batch size for indexing (default: 1000)
    #[arg(long)]
    pub batch: Option<usize>,

    /// Show index status only (don't index)
    #[arg(long)]
    pub status: bool,
}

/// Arguments for the search command.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Filter by note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Filter by tag
    #[arg(long, short)]
    pub tag: Option<String>,

    /// Exclude MOCs from search results
    #[arg(long)]
    pub exclude_mocs: bool,

    /// Filter by minimum value (0-100, default: 50)
    #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
    pub min_value: Option<u8>,

    /// Sort results by field: 'relevance' (default) or 'value'
    #[arg(long)]
    pub sort: Option<String>,
}

/// Arguments for the edit command.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// Note ID or file path
    pub id_or_path: String,

    /// Override default editor
    #[arg(long)]
    pub editor: Option<String>,
}

/// Arguments for the update command.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Note ID or file path
    pub id_or_path: String,

    /// Rename the note (updates filename)
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// Change note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Add one or more tags
    #[arg(long, action = clap::ArgAction::Append)]
    pub tag: Vec<String>,

    /// Remove one or more tags
    #[arg(long, action = clap::ArgAction::Append)]
    pub remove_tag: Vec<String>,

    /// Set the note's value score (0-100)
    #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
    pub value: Option<u8>,

    /// Update the source field
    #[arg(long)]
    pub source: Option<String>,

    /// Update the author field
    #[arg(long)]
    pub author: Option<String>,

    /// Update the generated_by field
    #[arg(long)]
    pub generated_by: Option<String>,

    /// Update the prompt_hash field
    #[arg(long)]
    pub prompt_hash: Option<String>,

    /// Update the verified flag
    #[arg(long, value_parser = crate::cli::parse::parse_bool)]
    pub verified: Option<bool>,
}
