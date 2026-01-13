//! `qipu init` command - create a new store
//!
//! Per spec (specs/cli-interface.md):
//! - Idempotent (safe to run multiple times)
//! - Non-interactive for agents
//! - Creates directory structure and default config

use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::store::{InitOptions, Store};

/// Execute the init command
pub fn execute(
    cli: &Cli,
    root: &Path,
    stealth: bool,
    visible: bool,
    branch: Option<String>,
) -> Result<()> {
    let options = InitOptions {
        visible,
        stealth,
        branch,
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

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "ok",
                "store": store.root().display().to_string(),
                "message": "Store initialized"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human | OutputFormat::Records => {
            println!("Initialized qipu store at {}", store.root().display());
        }
    }

    Ok(())
}
