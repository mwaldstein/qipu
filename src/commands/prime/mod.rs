//! `qipu prime` command - session-start primer for LLM agents
//!
//! Per spec (specs/llm-context.md):
//! - `qipu prime` outputs a short, bounded primer suitable for automatic injection
//!   at the start of an agent session.
//! - Requirements: deterministic ordering, stable formatting, bounded size (~1-2k tokens)
//! - Contents: qipu explanation, command reference, store location, key MOCs, recent notes
//!
//! MCP Mode: Detects if running in MCP/agent environment and outputs minimal primer (~50 tokens).
//! Detection methods: QIPU_MCP_MODE env var, MCP_SERVER env var, or MCP settings file.

pub mod budgeting;
pub mod mcp;
pub mod output;

use crate::cli::Cli;
use crate::commands::context::path_relative_to_cwd;
use qipu_core::error::Result;
use qipu_core::ontology::Ontology;
use qipu_core::store::Store;

use budgeting::{
    select_notes_within_budget, select_recent_within_budget, select_recent_within_budget_compact,
};
use mcp::detect_mcp_mode;
use output::{
    output_human, output_json, output_mcp_human, output_mcp_json, output_mcp_records,
    output_records,
};

/// Execute the prime command
pub fn execute(
    cli: &Cli,
    store: &Store,
    compact: bool,
    minimal: bool,
    full: bool,
    mcp: bool,
) -> Result<()> {
    let config = store.config();
    let ontology = Ontology::from_config_with_graph(&config.ontology, &config.graph);

    let notes = store.list_notes()?;
    let is_empty = notes.is_empty();

    // Determine output mode: MCP mode vs full mode
    // Priority: --mcp flag > --full flag > auto-detect > full mode (default)
    let use_mcp_mode = mcp || (!full && detect_mcp_mode());

    // Handle MCP mode output (~50 tokens)
    if use_mcp_mode {
        let store_path = path_relative_to_cwd(store.root());
        match cli.format {
            crate::cli::OutputFormat::Json => output_mcp_json(),
            crate::cli::OutputFormat::Human => output_mcp_human(&store_path),
            crate::cli::OutputFormat::Records => output_mcp_records(),
        }
        return Ok(());
    }

    // Full mode output (~1-2k tokens)
    let mut mocs: Vec<_> = notes.iter().filter(|n| n.note_type().is_moc()).collect();

    mocs.sort_by(
        |a, b| match (&b.frontmatter.updated, &a.frontmatter.updated) {
            (Some(b_updated), Some(a_updated)) => b_updated.cmp(a_updated),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        },
    );

    let top_mocs: Vec<_> = mocs.into_iter().collect();

    let mut recent_notes: Vec<_> = notes.iter().filter(|n| !n.note_type().is_moc()).collect();

    recent_notes.sort_by(|a, b| {
        let a_time = a
            .frontmatter
            .updated
            .as_ref()
            .or(a.frontmatter.created.as_ref());
        let b_time = b
            .frontmatter
            .updated
            .as_ref()
            .or(b.frontmatter.created.as_ref());
        match (b_time, a_time) {
            (Some(b_t), Some(a_t)) => b_t.cmp(a_t),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        }
    });

    let recent_notes: Vec<_> = recent_notes.into_iter().collect();

    let store_path = path_relative_to_cwd(store.root());

    let (selected_mocs, selected_recent) = if minimal {
        (vec![], vec![])
    } else if compact {
        (
            vec![],
            select_recent_within_budget_compact(&recent_notes, &store_path, cli.format, is_empty),
        )
    } else {
        let selected_mocs =
            select_notes_within_budget(&top_mocs, &recent_notes, &store_path, cli.format, is_empty);
        let selected_recent = select_recent_within_budget(
            &recent_notes,
            &selected_mocs,
            &store_path,
            cli.format,
            is_empty,
        );
        (selected_mocs, selected_recent)
    };

    match cli.format {
        crate::cli::OutputFormat::Json => {
            output_json(
                &store_path,
                &ontology,
                &config.ontology,
                &selected_mocs,
                &selected_recent,
                is_empty,
            )?;
        }
        crate::cli::OutputFormat::Human => {
            output_human(
                &store_path,
                &ontology,
                &config.ontology,
                &selected_mocs,
                &selected_recent,
                compact,
                is_empty,
            );
        }
        crate::cli::OutputFormat::Records => {
            output_records(
                &store_path,
                &ontology,
                &config.ontology,
                &selected_mocs,
                &selected_recent,
                is_empty,
            );
        }
    }

    Ok(())
}
