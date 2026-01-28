//! Main CLI commands enum and argument structures

use clap::Subcommand;
use std::path::PathBuf;

use crate::cli::args::CreateArgs;
use crate::cli::compact::CompactCommands;
use crate::cli::custom::CustomCommands;
use crate::cli::link::LinkCommands;
use crate::cli::ontology::OntologyCommands;
use crate::cli::parse::parse_note_type;
use crate::cli::store::StoreCommands;
use crate::cli::tags::TagsCommands;
use crate::cli::value::ValueCommands;
use crate::cli::workspace::WorkspaceCommands;
use crate::lib::note::NoteType;

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

        /// Skip automatic indexing
        #[arg(long)]
        no_index: bool,

        /// Override auto-indexing strategy (adaptive, full, incremental, quick)
        #[arg(long)]
        index_strategy: Option<String>,
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

        /// Filter by minimum value (0-100, default: 50)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        min_value: Option<u8>,

        /// Filter by custom metadata (format: key=value)
        #[arg(long)]
        custom: Option<String>,

        /// Include custom metadata in output (opt-in)
        #[arg(long)]
        show_custom: bool,
    },

    /// Show a note
    Show {
        /// Note ID or file path
        id_or_path: String,

        /// Show links for the note (inline + typed links, both directions)
        #[arg(long)]
        links: bool,

        /// Include custom metadata in output (opt-in)
        #[arg(long)]
        custom: bool,
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

        /// Resume from last checkpoint (for interrupted rebuild)
        #[arg(long)]
        resume: bool,

        /// Rewrite wiki-links to markdown links (optional; opt-in)
        #[arg(long)]
        rewrite_wiki_links: bool,

        /// Index only MOCs + recent 100 notes
        #[arg(long)]
        quick: bool,

        /// Index only notes with specified tag
        #[arg(long, short = 't')]
        tag: Option<String>,

        /// Index only notes of specified type
        #[arg(long, short = 'T', value_parser = parse_note_type)]
        r#type: Option<NoteType>,

        /// Index N most recently updated notes
        #[arg(long)]
        recent: Option<usize>,

        /// Index MOC and its linked notes
        #[arg(long)]
        moc: Option<String>,

        /// Show index status only (don't index)
        #[arg(long)]
        status: bool,
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

        /// Filter by minimum value (0-100, default: 50)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        min_value: Option<u8>,

        /// Sort results by field: 'relevance' (default) or 'value'
        #[arg(long)]
        sort: Option<String>,
    },

    /// Toggle verification status of a note
    Verify {
        /// Note ID or file path
        id_or_path: String,

        /// Explicitly set verification status (true/false)
        #[arg(long)]
        status: Option<bool>,
    },

    /// Manage note value (quality/importance score)
    Value {
        #[command(subcommand)]
        command: ValueCommands,
    },

    /// Manage and query tags
    Tags {
        #[command(subcommand)]
        command: TagsCommands,
    },

    /// Manage custom note metadata (for applications building on qipu)
    #[command(hide = true)]
    Custom {
        #[command(subcommand)]
        command: CustomCommands,
    },

    /// Manage and traverse note links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },

    /// Output session-start primer for LLM agents
    Prime {
        /// Compact output (omit MOCs and recent notes)
        #[arg(long)]
        compact: bool,

        /// Minimal output (only ontology and commands)
        #[arg(long)]
        minimal: bool,
    },

    /// Display minimal AGENTS.md snippet for agent integration
    Onboard,

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
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Check ontology (validates note/link types, warns on missing usage guidance)
        #[arg(long)]
        check: Option<Vec<String>>,
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
        /// Walk the graph from a note and bundle traversed notes
        #[arg(long)]
        walk: Option<String>,

        /// Direction for graph walk (out, in, or both)
        #[arg(long, default_value = "both")]
        walk_direction: String,

        /// Maximum traversal depth for graph walk
        #[arg(long, default_value = "3")]
        walk_max_hops: u32,

        /// Include only these link types in graph walk (can be repeated, or use CSV)
        #[arg(long, short = 'T', action = clap::ArgAction::Append, value_delimiter = ',')]
        walk_type: Vec<String>,

        /// Exclude these link types in graph walk (can be repeated, or use CSV)
        #[arg(long, action = clap::ArgAction::Append, value_delimiter = ',')]
        walk_exclude_type: Vec<String>,

        /// Show only typed links in graph walk (from frontmatter)
        #[arg(long)]
        walk_typed_only: bool,

        /// Show only inline links in graph walk (from markdown body)
        #[arg(long)]
        walk_inline_only: bool,

        /// Maximum nodes to visit in graph walk
        #[arg(long)]
        walk_max_nodes: Option<usize>,

        /// Maximum edges to emit in graph walk
        #[arg(long)]
        walk_max_edges: Option<usize>,

        /// Maximum neighbors per node in graph walk
        #[arg(long)]
        walk_max_fanout: Option<usize>,

        /// Filter by minimum value in graph walk (0-100, default: 50)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        walk_min_value: Option<u8>,

        /// Ignore note values during graph walk (unweighted BFS, weighted by default)
        #[arg(long, alias = "walk-unweighted", default_value = "false")]
        walk_ignore_value: bool,

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

        /// Include full note body content (default, use --summary-only to disable)
        #[arg(long, hide = true)]
        with_body: bool,

        /// Use summary instead of full body content
        #[arg(long)]
        summary_only: bool,

        /// Include safety banner for LLM prompt injection prevention
        #[arg(long)]
        safety_banner: bool,

        /// Add related notes using similarity expansion (threshold: 0.0-1.0, default: 0.3)
        ///
        /// Set to 0 to disable related-note expansion
        #[arg(long, default_value = "0.3")]
        related: f64,

        /// Include backlinks for selected notes
        #[arg(long)]
        backlinks: bool,

        /// Select notes by minimum value (0-100, can be used as standalone selector)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value, value_name = "N")]
        min_value: Option<u8>,

        /// Select notes by custom metadata (format: key=value, key, !key, key>n, key>=n, key<n, key<=n, can be used as standalone selector, can be repeated)
        #[arg(long, action = clap::ArgAction::Append)]
        custom_filter: Vec<String>,

        /// Include custom metadata in output (opt-in)
        #[arg(long)]
        custom: bool,
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

        /// Bibliography format: markdown, bibtex, csl-json (only used with --mode bibliography)
        #[arg(long, default_value = "markdown")]
        bib_format: String,

        /// Expand selection by traversing links (0 = no expansion)
        #[arg(long, default_value = "0")]
        max_hops: u32,

        /// Convert output to PDF using pandoc (requires pandoc to be installed)
        #[arg(long)]
        pdf: bool,
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

        /// Apply embedded config.toml from pack
        #[arg(long)]
        apply_config: bool,
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

    /// Open a note in $EDITOR and update the index upon completion
    Edit {
        /// Note ID or file path
        id_or_path: String,

        /// Override default editor
        #[arg(long)]
        editor: Option<String>,
    },

    /// Update a note's metadata or content non-interactively.
    /// Reads replacement body text from stdin when piped (e.g., `echo "new content" | qipu update <id>`)
    Update {
        /// Note ID or file path
        id_or_path: String,

        /// Rename the note (updates filename)
        #[arg(long, short = 't')]
        title: Option<String>,

        /// Change note type
        #[arg(long, short = 'T', value_parser = parse_note_type)]
        r#type: Option<NoteType>,

        /// Add one or more tags
        #[arg(long, action = clap::ArgAction::Append)]
        tag: Vec<String>,

        /// Remove one or more tags
        #[arg(long, action = clap::ArgAction::Append)]
        remove_tag: Vec<String>,

        /// Set the note's value score (0-100)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        value: Option<u8>,

        /// Update the source field
        #[arg(long)]
        source: Option<String>,

        /// Update the author field
        #[arg(long)]
        author: Option<String>,

        /// Update the generated_by field
        #[arg(long)]
        generated_by: Option<String>,

        /// Update the prompt_hash field
        #[arg(long)]
        prompt_hash: Option<String>,

        /// Update the verified flag
        #[arg(long, value_parser = crate::cli::parse::parse_bool)]
        verified: Option<bool>,
    },

    /// Manage the qipu store
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },

    /// Manage and display ontology configuration
    Ontology {
        #[command(subcommand)]
        command: OntologyCommands,
    },
}
