//! Shared output formatting helpers for commands
//!
//! Provides common patterns for JSON status messages, Records headers,
//! and other repeated formatting patterns across command modules.

pub mod ontology;
pub mod status;

pub use ontology::{print_ontology_human, print_ontology_json, print_ontology_records};
pub use status::{
    add_compaction_to_json, print_json_status, print_note_records, print_records_data,
    print_records_header, wrap_records_body,
};

use crate::cli::{Cli, OutputFormat};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::Note;

/// Compaction information for a note
pub struct CompactionInfo {
    pub count: usize,
    pub percentage: Option<f32>,
    pub compacted_ids: Vec<String>,
    pub truncated: bool,
}

/// Calculate compaction information for a note
pub fn calculate_compaction_info(
    cli: &Cli,
    note: &Note,
    note_map: &std::collections::HashMap<&str, &Note>,
    compaction_ctx: &CompactionContext,
) -> CompactionInfo {
    let count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    let percentage = compaction_ctx.get_compaction_pct(note, note_map);
    let (compacted_ids, truncated) = if cli.with_compaction_ids && count > 0 {
        let depth = cli.compaction_depth.unwrap_or(1);
        compaction_ctx
            .get_compacted_ids(&note.frontmatter.id, depth, cli.compaction_max_nodes)
            .unwrap_or((Vec::new(), false))
    } else {
        (Vec::new(), false)
    };

    CompactionInfo {
        count,
        percentage,
        compacted_ids,
        truncated,
    }
}

pub trait FormatDispatcher {
    fn output_json(&self) -> Result<()>;
    fn output_human(&self);
    fn output_records(&self);
}

pub fn dispatch_format<D: FormatDispatcher>(cli: &Cli, dispatcher: &D) -> Result<()> {
    match cli.format {
        OutputFormat::Json => dispatcher.output_json(),
        OutputFormat::Human => {
            dispatcher.output_human();
            Ok(())
        }
        OutputFormat::Records => {
            dispatcher.output_records();
            Ok(())
        }
    }
}
