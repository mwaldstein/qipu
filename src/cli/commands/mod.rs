//! Main CLI commands enum

use clap::Subcommand;

pub mod core;
pub mod data;
pub mod meta;

use crate::cli::args::CreateArgs;
use core::*;
use data::*;
use meta::*;

/// Top-level qipu commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new qipu store
    Init(InitArgs),

    /// Create a new note
    Create(CreateArgs),

    /// Alias for create
    New(CreateArgs),

    /// List notes
    List(ListArgs),

    /// Show a note
    Show(ShowArgs),

    /// List unprocessed notes (fleeting/literature)
    Inbox(InboxArgs),

    /// Create a new note from stdin
    Capture(CaptureArgs),

    /// Build or refresh derived indexes
    Index(IndexArgs),

    /// Search notes by title and body
    Search(SearchArgs),

    /// Open a note in $EDITOR and update the index upon completion
    Edit(EditArgs),

    /// Update a note's metadata or content non-interactively.
    /// Reads replacement body text from stdin when piped (e.g., `echo "new content" | qipu update <id>`)
    Update(UpdateArgs),

    /// Build context bundle for LLM integration
    Context(ContextArgs),

    /// Dump notes to a pack file
    Dump(DumpArgs),

    /// Export notes to a single document
    Export(ExportArgs),

    /// Load notes from a pack file
    Load(LoadArgs),

    /// Output session-start primer for LLM agents
    Prime(PrimeArgs),

    /// Toggle verification status of a note
    Verify(VerifyArgs),

    /// Manage note value (quality/importance score)
    Value(ValueSubcommand),

    /// Manage and query tags
    Tags(TagsSubcommand),

    /// Manage custom note metadata (for applications building on qipu)
    #[command(hide = true)]
    Custom(CustomSubcommand),

    /// Manage and traverse note links
    Link(LinkSubcommand),

    /// Display minimal AGENTS.md snippet for agent integration
    Onboard,

    /// Install qipu integration instructions for agent tools
    Setup(SetupArgs),

    /// Validate store invariants and optionally repair issues
    Doctor(DoctorArgs),

    /// Sync store: update indexes and optionally validate
    Sync(SyncArgs),

    /// Manage note compaction (digest-first navigation)
    Compact(CompactSubcommand),

    /// Manage and navigate isolated workspaces
    Workspace(WorkspaceSubcommand),

    /// Merge note id1 into id2
    Merge(MergeArgs),

    /// Manage the qipu store
    Store(StoreSubcommand),

    /// Manage and display ontology configuration
    Ontology(OntologySubcommand),

    /// Manage anonymous usage analytics
    Telemetry(TelemetrySubcommand),
}
