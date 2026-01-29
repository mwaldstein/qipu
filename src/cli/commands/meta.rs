//! Metadata and system management command argument structures

use clap::Args;

use crate::cli::compact::CompactCommands;
use crate::cli::custom::CustomCommands;
use crate::cli::link::LinkCommands;
use crate::cli::ontology::OntologyCommands;
use crate::cli::store::StoreCommands;
use crate::cli::tags::TagsCommands;
use crate::cli::value::ValueCommands;
use crate::cli::workspace::WorkspaceCommands;

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Note ID or file path
    pub id_or_path: String,

    /// Explicitly set verification status (true/false)
    #[arg(long)]
    pub status: Option<bool>,
}

#[derive(Args, Debug)]
pub struct ValueSubcommand {
    #[command(subcommand)]
    pub command: ValueCommands,
}

#[derive(Args, Debug)]
pub struct TagsSubcommand {
    #[command(subcommand)]
    pub command: TagsCommands,
}

#[derive(Args, Debug)]
pub struct CustomSubcommand {
    #[command(subcommand)]
    pub command: CustomCommands,
}

#[derive(Args, Debug)]
pub struct LinkSubcommand {
    #[command(subcommand)]
    pub command: LinkCommands,
}

#[derive(Args, Debug)]
pub struct SetupArgs {
    /// List available integrations
    #[arg(long)]
    pub list: bool,

    /// Tool/integration name (e.g., agents-md)
    pub tool: Option<String>,

    /// Print integration instructions to stdout
    #[arg(long)]
    pub print: bool,

    /// Check if integration is installed
    #[arg(long)]
    pub check: bool,

    /// Remove integration
    #[arg(long)]
    pub remove: bool,
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Auto-repair issues where possible
    #[arg(long)]
    pub fix: bool,

    /// Check for near-duplicate notes using similarity
    #[arg(long)]
    pub duplicates: bool,

    /// Similarity threshold for duplicate detection (0.0 to 1.0)
    #[arg(long, default_value = "0.85")]
    pub threshold: f64,

    /// Check ontology (validates note/link types, warns on missing usage guidance)
    #[arg(long)]
    pub check: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Run doctor validation after syncing
    #[arg(long)]
    pub validate: bool,

    /// Auto-repair issues if validation is enabled
    #[arg(long)]
    pub fix: bool,

    /// Commit changes to git if branch is configured
    #[arg(long)]
    pub commit: bool,

    /// Push changes to remote if branch is configured
    #[arg(long)]
    pub push: bool,
}

#[derive(Args, Debug)]
pub struct CompactSubcommand {
    #[command(subcommand)]
    pub command: CompactCommands,
}

#[derive(Args, Debug)]
pub struct WorkspaceSubcommand {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Args, Debug)]
pub struct MergeArgs {
    /// Source note ID (will be deleted)
    pub id1: String,

    /// Target note ID (will receive content and links)
    pub id2: String,

    /// Show what would happen without making changes
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct StoreSubcommand {
    #[command(subcommand)]
    pub command: StoreCommands,
}

#[derive(Args, Debug)]
pub struct OntologySubcommand {
    #[command(subcommand)]
    pub command: OntologyCommands,
}
