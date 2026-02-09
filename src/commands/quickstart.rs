//! Quickstart command - guided tour for common qipu workflows
//!
//! Per beads research (docs/research/beads-llm-bootstrapping.md):
//! - Shows common workflows: capture, link, search, context, session protocol
//! - Supports --format flag for json/records/human output

use crate::cli::Cli;
use crate::commands::dispatch::trace_command_always;
use crate::commands::format::{print_records_header, wrap_records_body};
use crate::output_by_format;
use qipu_core::error::Result;
use std::time::Instant;

const QUICKSTART_CONTENT: &str = r#"# Qipu Quick Start Guide

Welcome to qipu - a Zettelkasten-inspired knowledge management CLI.

## Capturing Knowledge

Capture quick insights from stdin:
  $ echo "Key finding..." | qipu capture --title "TIL: ..."

Create structured notes:
  $ qipu create "Paper: XYZ" --type literature --tag paper
  $ qipu create "Project Idea" --type fleeting --tag idea

## Building the Graph

Create typed links between notes:
  $ qipu link add <from-id> <to-id> --type derived-from
  $ qipu link add <from-id> <to-id> --type supports

Visualize connections:
  $ qipu link tree <note-id> --max-hops 2
  $ qipu link path <from-id> <to-id>

## Finding Knowledge

Search notes:
  $ qipu search "error handling"
  $ qipu search "rust" --format records

List and filter:
  $ qipu list --type permanent
  $ qipu inbox --exclude-linked

## Building Context

Create LLM context bundles:
  $ qipu context --note <id1> --note <id2> --max-chars 8000
  $ qipu context --tag rust --with-body --max-chars 16000

Explore from a starting point:
  $ qipu context --walk <note-id> --walk-max-hops 2

## Session Workflow

Start of session:
  $ qipu prime

During session:
  1. Capture insights as you work
  2. Link new notes to existing knowledge
  3. Use qipu context to build relevant context

End of session:
  $ git add .qipu && git commit -m "knowledge: captured insights"

Remember: Knowledge not committed is knowledge lost.

## Next Steps

Run `qipu prime` anytime for store overview.
Run `qipu --help` for complete command reference.
"#;

/// Execute the quickstart command
pub fn execute(cli: &Cli, start: Instant) -> Result<()> {
    output_by_format!(cli.format,
        json => {
            let output = serde_json::json!({
                "guide": QUICKSTART_CONTENT,
                "description": "Quick start guide for qipu workflows"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        },
        human => {
            print!("{}", QUICKSTART_CONTENT);
        },
        records => {
            print_records_header("quickstart", &[]);
            wrap_records_body("guide", QUICKSTART_CONTENT);
        }
    );
    trace_command_always!(start, "quickstart_execute");
    Ok(())
}
