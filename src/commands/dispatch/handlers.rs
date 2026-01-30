//! Local command handlers (not moved to other submodules)

use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::cli::{
    Cli, CustomCommands, OntologyCommands, StoreCommands, TagsCommands, ValueCommands,
};
use crate::commands::format::output_by_format_result;
use qipu_core::error::{QipuError, Result};
use tracing::debug;

use super::command::discover_or_open_store;

pub struct InitOptions {
    pub stealth: bool,
    pub visible: bool,
    pub branch: Option<String>,
    pub no_index: bool,
    pub index_strategy: Option<String>,
}

pub(super) fn handle_init(cli: &Cli, root: &Path, options: InitOptions) -> Result<()> {
    crate::commands::init::execute(
        cli,
        root,
        options.stealth,
        options.visible,
        options.branch,
        options.no_index,
        options.index_strategy,
    )
}

pub struct SetupOptions<'a> {
    pub list: bool,
    pub tool: Option<&'a str>,
    pub print: bool,
    pub check: bool,
    pub remove: bool,
}

pub(super) fn handle_setup(cli: &Cli, options: SetupOptions) -> Result<()> {
    crate::commands::setup::execute(
        cli,
        options.list,
        options.tool,
        options.print,
        options.check,
        options.remove,
    )
}

pub(super) fn handle_onboard(cli: &Cli) -> Result<()> {
    crate::commands::setup::execute_onboard(cli)
}

pub(super) fn handle_compact(cli: &Cli, command: &crate::cli::CompactCommands) -> Result<()> {
    crate::commands::compact::execute(cli, command)
}

pub(super) fn handle_workspace(cli: &Cli, command: &crate::cli::WorkspaceCommands) -> Result<()> {
    crate::commands::workspace::execute(cli, command)
}

pub(super) fn handle_value(
    cli: &Cli,
    root: &Path,
    command: &ValueCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;

    match command {
        ValueCommands::Set { id_or_path, score } => {
            if *score > 100 {
                return Err(QipuError::UsageError(
                    "Value score must be between 0 and 100".to_string(),
                ));
            }

            let mut note = if Path::new(id_or_path).exists() {
                let content = fs::read_to_string(id_or_path)?;
                qipu_core::note::Note::parse(&content, Some(id_or_path.into()))?
            } else {
                store.get_note(id_or_path)?
            };

            let note_id = note.id().to_string();

            note.frontmatter.value = Some(*score);

            store.save_note(&mut note)?;

            output_by_format_result!(cli.format,
                json => {
                    let output = serde_json::json!({
                        "id": note_id,
                        "value": score
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    Ok::<(), QipuError>(())
                },
                human => {
                    println!("{}: {}", note_id, score);
                },
                records => {
                    println!("T id=\"{}\" value={}", note_id, score);
                }
            )?;

            debug!(elapsed = ?start.elapsed(), "value_set");
            Ok(())
        }

        ValueCommands::Show { id_or_path } => {
            let note = if Path::new(id_or_path).exists() {
                let content = fs::read_to_string(id_or_path)?;
                qipu_core::note::Note::parse(&content, Some(id_or_path.into()))?
            } else {
                store.get_note(id_or_path)?
            };

            let note_id = note.id().to_string();
            let value = note.frontmatter.value.unwrap_or(50);
            let is_default = note.frontmatter.value.is_none();

            output_by_format_result!(cli.format,
                json => {
                    let output = serde_json::json!({
                        "id": note_id,
                        "value": value,
                        "default": is_default
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    Ok::<(), QipuError>(())
                },
                human => {
                    if is_default {
                        println!("{}: {} (default)", note_id, value);
                    } else {
                        println!("{}: {}", note_id, value);
                    }
                },
                records => {
                    println!(
                        "T id=\"{}\" value={} default={}",
                        note_id, value, is_default
                    );
                }
            )?;

            debug!(elapsed = ?start.elapsed(), "value_show");
            Ok(())
        }
    }
}

pub(super) fn handle_tags(
    cli: &Cli,
    root: &Path,
    command: &TagsCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;

    match command {
        TagsCommands::List {} => {
            let frequencies = store.get_tag_frequencies()?;

            output_by_format_result!(cli.format,
                json => {
                    let output: Vec<_> = frequencies
                        .iter()
                        .map(|(tag, count)| {
                            serde_json::json!({
                                "tag": tag,
                                "count": count
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    Ok::<(), QipuError>(())
                },
                human => {
                    if frequencies.is_empty() {
                        if !cli.quiet {
                            println!("No tags found");
                        }
                    } else {
                        for (tag, count) in &frequencies {
                            println!("{}: {}", tag, count);
                        }
                    }
                },
                records => {
                    if frequencies.is_empty() {
                        if !cli.quiet {
                            println!("No tags found");
                        }
                    } else {
                        for (tag, count) in &frequencies {
                            println!("T tag=\"{}\" count={}", tag, count);
                        }
                    }
                }
            )?;

            debug!(elapsed = ?start.elapsed(), "tags_list");
            Ok(())
        }
    }
}

pub(super) fn handle_custom(
    cli: &Cli,
    root: &Path,
    command: &CustomCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    let format = cli.format;

    match command {
        CustomCommands::Set {
            id_or_path,
            key,
            value,
        } => {
            crate::commands::custom::set_custom_field(
                &store, id_or_path, key, value, format, cli.quiet,
            )?;
            debug!(elapsed = ?start.elapsed(), "custom_set");
            Ok(())
        }

        CustomCommands::Get { id_or_path, key } => {
            crate::commands::custom::get_custom_field(&store, id_or_path, key, format)?;
            debug!(elapsed = ?start.elapsed(), "custom_get");
            Ok(())
        }

        CustomCommands::Show { id_or_path } => {
            crate::commands::custom::show_custom_fields(&store, id_or_path, format)?;
            debug!(elapsed = ?start.elapsed(), "custom_show");
            Ok(())
        }

        CustomCommands::Unset { id_or_path, key } => {
            crate::commands::custom::unset_custom_field(
                &store, id_or_path, key, format, cli.quiet,
            )?;
            debug!(elapsed = ?start.elapsed(), "custom_unset");
            Ok(())
        }
    }
}

pub(super) fn handle_store(
    cli: &Cli,
    root: &Path,
    command: &StoreCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;

    match command {
        StoreCommands::Stats {} => {
            crate::commands::store::execute_stats(cli, &store)?;
            debug!(elapsed = ?start.elapsed(), "store_stats");
            Ok(())
        }
    }
}

pub(super) fn handle_ontology(
    cli: &Cli,
    root: &Path,
    command: &OntologyCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;

    match command {
        OntologyCommands::Show {} => {
            crate::commands::ontology::execute_show(cli, &store)?;
            debug!(elapsed = ?start.elapsed(), "ontology_show");
            Ok(())
        }
    }
}
