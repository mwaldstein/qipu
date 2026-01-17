//! `qipu sync` command - synchronize indexes and optionally validate
//!
//! Per spec (specs/cli-interface.md):
//! - Ensure derived indexes are up to date
//! - Optionally run validations
//! - Optional convenience command for multi-agent workflows

use crate::cli::{Cli, OutputFormat};
use crate::commands::doctor;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

/// Execute the sync command
pub fn execute(
    cli: &Cli,
    store: &Store,
    validate: bool,
    fix: bool,
    commit: bool,
    push: bool,
) -> Result<()> {
    // Step 1: Update indexes silently
    let builder = IndexBuilder::new(store);
    let builder = builder.load_existing()?;
    let index = builder.build()?;

    // Save index to cache
    let cache_dir = store.root().join(".cache");
    index.save(&cache_dir)?;

    let notes_indexed = index.metadata.len();
    let tags_indexed = index.tags.len();
    let edges_indexed = index.edges.len();

    // Step 2: Handle git automation if branch is configured
    if let Some(branch_name) = &store.config().branch {
        if commit || push {
            use crate::lib::git;

            // Determine repository root (assume store parent for now)
            let repo_root = store.root().parent().ok_or_else(|| {
                crate::lib::error::QipuError::Other("Cannot determine repository root".to_string())
            })?;

            if !git::is_git_available() {
                return Err(crate::lib::error::QipuError::Other(
                    "Git not found in PATH".to_string(),
                ));
            }

            // Setup branch workflow (switch to qipu branch)
            let original_branch = git::setup_branch_workflow(repo_root, branch_name)?;

            let result = (|| -> Result<()> {
                // Commit if requested and there are changes
                if commit {
                    if git::has_changes(repo_root)? {
                        git::add(repo_root, ".")?;
                        git::commit(repo_root, "qipu sync: update notes and indexes")?;
                    }
                }

                // Push if requested
                if push {
                    git::push(repo_root, "origin", branch_name)?;
                }

                Ok(())
            })();

            // Always attempt to switch back to the original branch
            let checkout_result = git::checkout_branch(repo_root, &original_branch);

            // Return the first error encountered
            result?;
            checkout_result?;
        }
    }

    // Step 3: Optionally validate
    let (errors, warnings, fixed) = if validate {
        // Run doctor quietly - it will output its own results
        let result = doctor::execute(cli, store, fix, false, 0.8)?;
        (result.error_count, result.warning_count, result.fixed_count)
    } else {
        (0, 0, 0)
    };

    // Output based on format - but only if doctor wasn't run or we're in human mode
    // In JSON/Records mode, doctor will output its own structured result
    if !validate || cli.format == OutputFormat::Human {
        match cli.format {
            OutputFormat::Human => {
                if !cli.quiet {
                    println!("Indexed {} notes", notes_indexed);
                    if validate {
                        println!("Store validated: {} errors, {} warnings", errors, warnings);
                        if fixed > 0 {
                            println!("Fixed {} issues", fixed);
                        }
                    }
                }
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "ok",
                    "notes_indexed": notes_indexed,
                    "tags_indexed": tags_indexed,
                    "edges_indexed": edges_indexed,
                    "validation": if validate {
                        serde_json::json!({
                            "errors": errors,
                            "warnings": warnings,
                            "fixed": fixed,
                        })
                    } else {
                        serde_json::Value::Null
                    },
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            OutputFormat::Records => {
                // Header line per spec (specs/records-output.md)
                let mut header = format!(
                    "H qipu=1 records=1 store={} mode=sync notes={} tags={} edges={}",
                    store.root().display(),
                    notes_indexed,
                    tags_indexed,
                    edges_indexed
                );
                if validate {
                    header.push_str(&format!(
                        " errors={} warnings={} fixed={}",
                        errors, warnings, fixed
                    ));
                }
                println!("{}", header);
            }
        }
    }

    Ok(())
}
