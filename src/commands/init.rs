//! `qipu init` command - create a new store
//!
//! Per spec (specs/cli-interface.md):
//! - Idempotent (safe to run multiple times)
//! - Non-interactive for agents
//! - Creates directory structure and default config

use std::path::Path;

use crate::cli::Cli;
use crate::commands::format::output_by_format_result;
use qipu_core::error::Result;
use qipu_core::store::{InitOptions, Store};

/// Execute the init command
pub fn execute(
    cli: &Cli,
    root: &Path,
    stealth: bool,
    visible: bool,
    branch: Option<String>,
    no_index: bool,
    index_strategy: Option<String>,
) -> Result<()> {
    let options = InitOptions {
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
            println!();
            println!("Run `qipu prime` for workflow context.");
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
