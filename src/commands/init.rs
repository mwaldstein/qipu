//! `qipu init` command - create a new store
//!
//! Per spec (specs/cli-interface.md):
//! - Idempotent (safe to run multiple times)
//! - Non-interactive for agents
//! - Creates directory structure and default config

use std::io::Write;
use std::path::Path;

use crate::cli::Cli;
use crate::commands::format::output_by_format_result;
use qipu_core::error::Result;
use qipu_core::store::{InitOptions as StoreInitOptions, Store};

/// Minimal qipu section for AGENTS.md
const QIPU_AGENTS_SECTION: &str = r#"## Qipu Knowledge

This project uses **qipu** for knowledge management.
Run `qipu prime` for workflow context.

**Quick reference:**
- `qipu prime` - Get store overview
- `qipu create` - Create note
- `qipu capture` - Quick capture
- `qipu search` - Search notes
- `qipu context` - Build LLM context

For full workflow: `qipu prime`
"#;

fn write_agents_md(root: &Path, verbose: bool) -> Result<()> {
    let agents_md_path = root.join("AGENTS.md");

    if !agents_md_path.exists() {
        // Create new AGENTS.md with qipu section
        let content = format!("# Agent Instructions\n\n{}", QIPU_AGENTS_SECTION);
        std::fs::write(&agents_md_path, content)?;
        if verbose {
            println!("  Created AGENTS.md with qipu instructions");
        }
    } else {
        // Append if "## Qipu Knowledge" section not present
        let existing = std::fs::read_to_string(&agents_md_path)?;
        if !existing.contains("## Qipu Knowledge") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&agents_md_path)?;
            writeln!(file, "\n{}", QIPU_AGENTS_SECTION)?;
            if verbose {
                println!("  Added qipu section to AGENTS.md");
            }
        } else if verbose {
            println!("  AGENTS.md already contains qipu section");
        }
    }

    Ok(())
}

/// Execute the init command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    root: &Path,
    stealth: bool,
    visible: bool,
    branch: Option<String>,
    no_index: bool,
    index_strategy: Option<String>,
    agents_md: bool,
) -> Result<()> {
    let options = StoreInitOptions {
        visible,
        stealth,
        branch,
        no_index,
        index_strategy,
    };

    let store = if let Some(path) = cli.store.as_ref() {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::init_at(&resolved, options, Some(root))?
    } else {
        Store::init(root, options)?
    };

    // Optionally write qipu section to AGENTS.md
    if agents_md {
        write_agents_md(root, cli.verbose)?;
    }

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "status": "ok",
                "store": store.root().display().to_string(),
                "message": "Store initialized"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok::<(), qipu_core::error::QipuError>(())
        },
        human => {
            println!("Initialized qipu store at {}", store.root().display());
            if agents_md {
                println!();
                println!("Run `qipu prime` for workflow context.");
            } else {
                println!();
                println!("Run `qipu prime` for workflow context.");
                println!("Run `qipu setup agents-md` to add instructions to AGENTS.md.");
            }
        },
        records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=init status=ok",
                store.root().display()
            );
        }
    )?;

    Ok(())
}
