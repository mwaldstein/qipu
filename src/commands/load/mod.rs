//! `qipu load` command - load notes from a pack file
//!
//! Per spec (specs/pack.md):
//! - Load pack file into store
//! - Restore notes, links, and attachments
//! - No content transformation
//! - Handle merge semantics for loading into non-empty stores

pub mod deserialize;
pub mod loader;
pub mod metadata;
pub mod model;
pub mod parsers;

use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use loader::{load_attachments, load_links, load_notes};
use qipu_core::bail_invalid;
use qipu_core::bail_unsupported;
use qipu_core::config::STORE_FORMAT_VERSION;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

pub(crate) enum LoadStrategy {
    Skip,
    Overwrite,
    MergeLinks,
}

fn parse_strategy(s: &str) -> Result<LoadStrategy> {
    match s.to_lowercase().as_str() {
        "skip" => Ok(LoadStrategy::Skip),
        "overwrite" => Ok(LoadStrategy::Overwrite),
        "merge-links" => Ok(LoadStrategy::MergeLinks),
        _ => Err(QipuError::unsupported(
            "strategy",
            s,
            "skip, overwrite, merge-links",
        )),
    }
}

pub fn execute(
    cli: &Cli,
    store: &Store,
    pack_file: &Path,
    strategy: &str,
    apply_config: bool,
) -> Result<()> {
    let strategy = parse_strategy(strategy)?;

    let pack_content = std::fs::read_to_string(pack_file)
        .map_err(|e| QipuError::io_operation("read", "pack file", e))?;

    let pack_data = if deserialize::looks_like_json(&pack_content) {
        deserialize::parse_json_pack(&pack_content)?
    } else {
        deserialize::parse_records_pack(&pack_content)?
    };

    if pack_data.header.version != "1.0" {
        bail_unsupported!("pack version", &pack_data.header.version, "1.0");
    }

    if pack_data.header.store_version > STORE_FORMAT_VERSION {
        bail_invalid!(
            &format!("pack store version {}", pack_data.header.store_version),
            format!(
                "higher than store version {} - please upgrade qipu",
                STORE_FORMAT_VERSION
            )
        );
    }

    if apply_config {
        if !pack_data.config_content.is_empty() {
            let config_path = store.config_path();
            std::fs::write(&config_path, &pack_data.config_content)
                .map_err(|e| QipuError::io_operation("write", "config.toml", e))?;
            tracing::info!("Applied config from pack to {}", config_path.display());
        } else {
            tracing::warn!("Pack contains no config to apply");
        }
    }

    let (loaded_notes_count, loaded_ids, new_ids) = load_notes(store, &pack_data.notes, &strategy)?;

    let all_existing_ids = store.existing_ids()?;
    let loaded_links_count = match strategy {
        LoadStrategy::Skip => load_links(
            store,
            &pack_data.links,
            &new_ids,
            &new_ids,
            &all_existing_ids,
        )?,
        LoadStrategy::MergeLinks => load_links(
            store,
            &pack_data.links,
            &loaded_ids,
            &new_ids,
            &all_existing_ids,
        )?,
        LoadStrategy::Overwrite => load_links(
            store,
            &pack_data.links,
            &loaded_ids,
            &loaded_ids,
            &all_existing_ids,
        )?,
    };

    let loaded_attachments_count = if !pack_data.attachments.is_empty() {
        load_attachments(store, &pack_data.attachments, &pack_data.notes)?
    } else {
        0
    };

    tracing::debug!(
        notes_loaded = loaded_notes_count,
        links_loaded = loaded_links_count,
        attachments_loaded = loaded_attachments_count,
        pack_file = %pack_file.display(),
        "Load completed"
    );

    match cli.format {
        OutputFormat::Human => {
            // Human format output is handled by the tracing event above
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "pack_file": pack_file.display().to_string(),
                "notes_loaded": loaded_notes_count,
                "links_loaded": loaded_links_count,
                "attachments_loaded": loaded_attachments_count,
                "pack_info": pack_data.header,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Records => {
            let store_path = store.root().display().to_string();
            println!(
                "H qipu=1 records=1 store={} mode=load pack_file={} notes={} links={} attachments={}",
                store_path,
                pack_file.display(),
                loaded_notes_count,
                loaded_links_count,
                loaded_attachments_count
            );
        }
    }

    Ok(())
}
