use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use qipu_core::compaction::CompactionContext;
use qipu_core::note::Note;
use qipu_core::store::Store;

pub mod bibliography;
pub mod bundle;
pub mod json;
pub mod links;
pub mod markdown_utils;
pub mod outline;
pub mod records;

// Re-export the public API
pub use bibliography::export_bibliography;
pub use bundle::export_bundle;
pub use json::export_json;
pub use outline::export_outline;
pub use records::export_records;

// Re-export ExportMode and LinkMode from parent
pub use super::{ExportMode, LinkMode};

/// Shared context for export functions
pub struct ExportContext<'a> {
    pub notes: &'a [Note],
    pub store: &'a Store,
    pub options: &'a ExportOptions<'a>,
    pub cli: &'a Cli,
    pub compaction_ctx: &'a CompactionContext,
    pub all_notes: &'a [Note],
}
