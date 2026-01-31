//! Local command handlers (not moved to other submodules)

use std::path::Path;
use std::time::Instant;

use crate::cli::{
    Cli, CompactCommands, CustomCommands, OntologyCommands, StoreCommands, TagsCommands,
    ValueCommands, WorkspaceCommands,
};
use crate::commands::format::output_by_format_result;
use qipu_core::bail_usage;
use qipu_core::error::{QipuError, Result};

use super::command::discover_or_open_store;
use super::trace_command_always;

pub struct InitOptions<'a> {
    pub stealth: bool,
    pub visible: bool,
    pub branch: Option<&'a str>,
    pub no_index: bool,
    pub index_strategy: Option<&'a str>,
}

pub(super) fn handle_init(cli: &Cli, root: &Path, options: InitOptions) -> Result<()> {
    crate::commands::init::execute(
        cli,
        root,
        options.stealth,
        options.visible,
        options.branch.map(|s| s.to_string()),
        options.no_index,
        options.index_strategy.map(|s| s.to_string()),
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

pub(super) fn handle_compact(
    cli: &Cli,
    root: &Path,
    command: &crate::cli::CompactCommands,
) -> Result<()> {
    crate::commands::compact::execute(cli, root, command)
}

pub(super) fn handle_workspace(
    cli: &Cli,
    root: &Path,
    command: &crate::cli::WorkspaceCommands,
) -> Result<()> {
    crate::commands::workspace::execute(cli, root, command)
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
                bail_usage!("Value score must be between 0 and 100");
            }

            let mut note = store.load_note_by_id_or_path(id_or_path)?;

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

            trace_command_always!(start, "value_set");
            Ok(())
        }

        ValueCommands::Show { id_or_path } => {
            let note = store.load_note_by_id_or_path(id_or_path)?;

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

            trace_command_always!(start, "value_show");
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

            trace_command_always!(start, "tags_list");
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
            trace_command_always!(start, "custom_set");
            Ok(())
        }

        CustomCommands::Get { id_or_path, key } => {
            crate::commands::custom::get_custom_field(&store, id_or_path, key, format)?;
            trace_command_always!(start, "custom_get");
            Ok(())
        }

        CustomCommands::Show { id_or_path } => {
            crate::commands::custom::show_custom_fields(&store, id_or_path, format)?;
            trace_command_always!(start, "custom_show");
            Ok(())
        }

        CustomCommands::Unset { id_or_path, key } => {
            crate::commands::custom::unset_custom_field(
                &store, id_or_path, key, format, cli.quiet,
            )?;
            trace_command_always!(start, "custom_unset");
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
            trace_command_always!(start, "store_stats");
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
            trace_command_always!(start, "ontology_show");
            Ok(())
        }
    }
}

pub(super) fn execute_value(
    cli: &Cli,
    root: &Path,
    command: &ValueCommands,
    start: Instant,
) -> Result<()> {
    handle_value(cli, root, command, start)
}

pub(super) fn execute_tags(
    cli: &Cli,
    root: &Path,
    command: &TagsCommands,
    start: Instant,
) -> Result<()> {
    handle_tags(cli, root, command, start)
}

pub(super) fn execute_custom(
    cli: &Cli,
    root: &Path,
    command: &CustomCommands,
    start: Instant,
) -> Result<()> {
    handle_custom(cli, root, command, start)
}

pub(super) fn execute_onboard(cli: &Cli) -> Result<()> {
    handle_onboard(cli)
}

pub(super) fn execute_setup(cli: &Cli, options: SetupOptions) -> Result<()> {
    handle_setup(cli, options)
}

pub(super) fn execute_compact(cli: &Cli, root: &Path, command: &CompactCommands) -> Result<()> {
    handle_compact(cli, root, command)
}

pub(super) fn execute_workspace(cli: &Cli, root: &Path, command: &WorkspaceCommands) -> Result<()> {
    handle_workspace(cli, root, command)
}

pub(super) fn execute_store(
    cli: &Cli,
    root: &Path,
    command: &StoreCommands,
    start: Instant,
) -> Result<()> {
    handle_store(cli, root, command, start)
}

pub(super) fn execute_ontology_dispatch(
    cli: &Cli,
    root: &Path,
    command: &OntologyCommands,
    start: Instant,
) -> Result<()> {
    handle_ontology(cli, root, command, start)
}
