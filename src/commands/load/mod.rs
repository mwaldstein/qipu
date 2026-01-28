//! `qipu load` command - load notes from a pack file
//!
//! Per spec (specs/pack.md):
//! - Load pack file into store
//! - Restore notes, links, and attachments
//! - No content transformation
//! - Handle merge semantics for loading into non-empty stores

pub mod deserialize;
pub mod model;

use std::collections::HashSet;
use std::path::Path;

use base64::{engine::general_purpose, Engine as _};

use crate::cli::{Cli, OutputFormat};
use crate::lib::config::STORE_FORMAT_VERSION;
use crate::lib::error::{QipuError, Result};
use crate::lib::note::{Note, NoteFrontmatter, NoteType, Source};
use crate::lib::store::Store;
use model::{PackAttachment, PackLink, PackNote};

enum LoadStrategy {
    Skip,
    Overwrite,
    MergeLinks,
}

/// Convert serde_json::Value to serde_yaml::Value
fn serde_json_to_yaml(json: &serde_json::Value) -> serde_yaml::Value {
    match json {
        serde_json::Value::Null => serde_yaml::Value::Null,
        serde_json::Value::Bool(b) => serde_yaml::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(i))
            } else if let Some(u) = n.as_u64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(u))
            } else if let Some(f) = n.as_f64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(f))
            } else {
                serde_yaml::Value::Null
            }
        }
        serde_json::Value::String(s) => serde_yaml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            serde_yaml::Value::Sequence(arr.iter().map(serde_json_to_yaml).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: serde_yaml::Mapping = obj
                .iter()
                .map(|(k, v)| (serde_yaml::Value::String(k.clone()), serde_json_to_yaml(v)))
                .collect();
            serde_yaml::Value::Mapping(map)
        }
    }
}

fn parse_strategy(s: &str) -> Result<LoadStrategy> {
    match s.to_lowercase().as_str() {
        "skip" => Ok(LoadStrategy::Skip),
        "overwrite" => Ok(LoadStrategy::Overwrite),
        "merge-links" => Ok(LoadStrategy::MergeLinks),
        _ => Err(QipuError::Other(format!(
            "invalid strategy: {} (valid: skip, overwrite, merge-links)",
            s
        ))),
    }
}

/// Execute load command
pub fn execute(
    cli: &Cli,
    store: &Store,
    pack_file: &Path,
    strategy: &str,
    apply_config: bool,
) -> Result<()> {
    // Parse strategy
    let strategy = parse_strategy(strategy)?;

    // Read pack file
    let pack_content = std::fs::read_to_string(pack_file)
        .map_err(|e| QipuError::Other(format!("failed to read pack file: {}", e)))?;

    // Parse pack content based on format
    let pack_data = if deserialize::looks_like_json(&pack_content) {
        deserialize::parse_json_pack(&pack_content)?
    } else {
        deserialize::parse_records_pack(&pack_content)?
    };

    // Validate pack version
    if pack_data.header.version != "1.0" {
        return Err(QipuError::Other(format!(
            "unsupported pack version: {} (supported: 1.0)",
            pack_data.header.version
        )));
    }

    // Validate store version compatibility
    if pack_data.header.store_version > STORE_FORMAT_VERSION {
        return Err(QipuError::Other(format!(
            "pack store version {} is higher than store version {} - please upgrade qipu",
            pack_data.header.store_version, STORE_FORMAT_VERSION
        )));
    }

    // Apply config if requested and config is present
    if apply_config {
        if !pack_data.config_content.is_empty() {
            let config_path = store.root().join("config.toml");
            std::fs::write(&config_path, &pack_data.config_content)
                .map_err(|e| QipuError::Other(format!("failed to write config.toml: {}", e)))?;
            tracing::info!("Applied config from pack to {}", config_path.display());
        } else {
            tracing::warn!("Pack contains no config to apply");
        }
    }

    // Load notes
    let (loaded_notes_count, loaded_ids, new_ids) = load_notes(store, &pack_data.notes, &strategy)?;

    // Load links
    // Get current existing IDs (including newly loaded notes) for edge resolution
    let all_existing_ids = store.existing_ids()?;
    // Choose which IDs to use based on strategy:
    // - skip: only load links between newly loaded notes
    // - merge-links: load links from loaded notes (new or existing) but only to newly loaded notes
    // - overwrite: load links from loaded notes to loaded notes
    let loaded_links_count = match strategy {
        LoadStrategy::Skip => {
            // Only load links between newly loaded notes
            load_links(
                store,
                &pack_data.links,
                &new_ids,
                &new_ids,
                &all_existing_ids,
            )?
        }
        LoadStrategy::MergeLinks => {
            // Load links from loaded notes (new or existing), but only to newly loaded notes
            load_links(
                store,
                &pack_data.links,
                &loaded_ids,
                &new_ids,
                &all_existing_ids,
            )?
        }
        LoadStrategy::Overwrite => {
            // Load links from and to loaded notes
            load_links(
                store,
                &pack_data.links,
                &loaded_ids,
                &loaded_ids,
                &all_existing_ids,
            )?
        }
    };

    // Load attachments
    let loaded_attachments_count = if !pack_data.attachments.is_empty() {
        load_attachments(store, &pack_data.attachments, &pack_data.notes)?
    } else {
        0
    };

    // Report results
    tracing::debug!(
        notes_loaded = loaded_notes_count,
        links_loaded = loaded_links_count,
        attachments_loaded = loaded_attachments_count,
        pack_file = %pack_file.display(),
        "Load completed"
    );

    // Output in requested format if needed
    match cli.format {
        OutputFormat::Human => {
            // Already reported above
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

fn write_note_preserving_updated(
    store: &Store,
    note: &Note,
    existing_ids: &HashSet<String>,
) -> Result<()> {
    let path = note
        .path
        .as_ref()
        .ok_or_else(|| QipuError::Other("cannot save note without path".to_string()))?;
    let new_content = note.to_markdown()?;

    let should_write = if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(existing) => existing != new_content,
            Err(_) => true,
        }
    } else {
        true
    };

    if should_write {
        std::fs::write(path, new_content)?;

        // Update database after file write to maintain consistency
        store.db().insert_note(note)?;
        // Also insert edges to ensure links are stored in the database
        store.db().insert_edges(note, existing_ids)?;
    }

    Ok(())
}

/// Load notes from pack into store
fn load_notes(
    store: &Store,
    pack_notes: &[PackNote],
    strategy: &LoadStrategy,
) -> Result<(usize, HashSet<String>, HashSet<String>)> {
    let mut loaded_count = 0;
    let mut loaded_ids: HashSet<String> = HashSet::new();
    let mut new_ids: HashSet<String> = HashSet::new();
    let existing_ids = store.existing_ids()?;

    for pack_note in pack_notes {
        // Parse note type
        let note_type = pack_note.note_type.parse::<NoteType>().map_err(|e| {
            QipuError::Other(format!(
                "invalid note type '{}': {}",
                pack_note.note_type, e
            ))
        })?;

        // Parse sources
        let sources = pack_note
            .sources
            .iter()
            .map(|s| Source {
                url: s.url.clone(),
                title: s.title.clone(),
                accessed: s.accessed.as_ref().and_then(|s| s.parse().ok()),
            })
            .collect();

        // Create note frontmatter
        let frontmatter = NoteFrontmatter {
            id: pack_note.id.clone(),
            title: pack_note.title.clone(),
            note_type: Some(note_type.clone()),
            tags: pack_note.tags.clone(),
            created: pack_note.created,
            updated: pack_note.updated,
            sources,
            links: Vec::new(),
            summary: pack_note.summary.clone(),
            compacts: pack_note.compacts.clone(),
            source: pack_note.source.clone(),
            author: pack_note.author.clone(),
            generated_by: pack_note.generated_by.clone(),
            prompt_hash: pack_note.prompt_hash.clone(),
            verified: pack_note.verified,
            value: pack_note.value,
            custom: pack_note
                .custom
                .iter()
                .map(|(k, v)| (k.clone(), serde_json_to_yaml(v)))
                .collect(),
        };

        // Create note with path from pack if available
        let pack_path = pack_note.path.as_ref().map(|p| {
            let path = std::path::Path::new(p);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                // Resolve relative paths against store root
                store.root().join(path)
            }
        });

        let mut note = Note {
            frontmatter,
            body: pack_note.content.clone(),
            path: pack_path.clone(),
        };

        // Determine target directory for fallback path generation
        let target_dir = if note_type.is_moc() {
            store.mocs_dir()
        } else {
            store.notes_dir()
        };

        // Handle conflicts based on strategy
        let should_load = if existing_ids.contains(note.id()) {
            // For overwrite strategy, get the existing note's path to overwrite it in place
            if matches!(strategy, LoadStrategy::Overwrite) {
                if let Ok(existing_note) = store.get_note(note.id()) {
                    if let Some(existing_path) = existing_note.path {
                        note.path = Some(existing_path);
                    }
                }
            }

            // Determine target directory and filename if not already set
            // Only generate a path if pack didn't provide one
            if note.path.is_none() {
                let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
                note.path = Some(target_dir.join(&file_name));
            }

            match strategy {
                LoadStrategy::Skip => {
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Skipping conflicting note"
                    );
                    false
                }
                LoadStrategy::Overwrite => {
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Overwriting existing note"
                    );
                    // Delete the old file first to ensure clean overwrite
                    if let Some(path) = &note.path {
                        if path.exists() {
                            std::fs::remove_file(path).map_err(|e| {
                                QipuError::Other(format!(
                                    "failed to delete existing file {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;
                        }
                    }
                    true
                }
                LoadStrategy::MergeLinks => {
                    // Keep existing note content, only add links from pack
                    // Return false to skip writing pack note content, but note will still be added to loaded_ids
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Keeping existing note content, will merge links from pack"
                    );
                    false
                }
            }
        } else {
            // New note - determine filename if not provided by pack
            if note.path.is_none() {
                let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
                note.path = Some(target_dir.join(&file_name));
            }
            true
        };

        if should_load {
            // Save note without overwriting pack timestamps
            write_note_preserving_updated(store, &note, &existing_ids)?;
            loaded_count += 1;
            loaded_ids.insert(note.id().to_string());
            // Track newly loaded notes (notes that didn't exist before)
            if !existing_ids.contains(note.id()) {
                new_ids.insert(note.id().to_string());
            }
        } else if matches!(strategy, LoadStrategy::MergeLinks) && existing_ids.contains(note.id()) {
            // For merge-links, add existing note to loaded_ids so links will be processed
            loaded_ids.insert(note.id().to_string());
        }
    }

    Ok((loaded_count, loaded_ids, new_ids))
}

/// Load links from pack into store
///
/// # Arguments
/// * `source_ids` - Only process links FROM notes in this set
/// * `target_ids` - Only add links TO notes in this set
fn load_links(
    store: &Store,
    pack_links: &[PackLink],
    source_ids: &HashSet<String>,
    target_ids: &HashSet<String>,
    existing_ids: &HashSet<String>,
) -> Result<usize> {
    let mut loaded_count = 0;

    // Group links by source note to batch process
    let mut links_by_source: std::collections::HashMap<String, Vec<PackLink>> =
        std::collections::HashMap::new();
    for pack_link in pack_links {
        links_by_source
            .entry(pack_link.from.clone())
            .or_default()
            .push(pack_link.clone());
    }

    for (source_id, links) in links_by_source {
        // Only process links for notes that are in source_ids
        if source_ids.contains(&source_id) {
            // Load the source note
            let mut source_note = store.get_note(&source_id)?;

            // Add each link to the note's frontmatter
            for pack_link in links {
                // Only load links where target note is in target_ids
                if target_ids.contains(&pack_link.to) {
                    // Check if link already exists to avoid duplicates
                    let link_exists = source_note.frontmatter.links.iter().any(|l| {
                        l.id == pack_link.to.as_str()
                            && l.link_type == pack_link.link_type.as_deref().unwrap_or("")
                    });

                    if !link_exists {
                        // Parse link type if present
                        if let Some(ref type_str) = pack_link.link_type {
                            let link_type = type_str.parse().map_err(|e| {
                                QipuError::Other(format!("invalid link type '{}': {}", type_str, e))
                            })?;

                            source_note
                                .frontmatter
                                .links
                                .push(crate::lib::note::TypedLink {
                                    link_type,
                                    id: pack_link.to.clone(),
                                });
                            loaded_count += 1;
                        }
                    }
                }
            }

            write_note_preserving_updated(store, &source_note, existing_ids)?;
        }
    }

    Ok(loaded_count)
}

/// Load attachments from pack into store
fn load_attachments(
    store: &Store,
    pack_attachments: &[PackAttachment],
    _loaded_notes: &[PackNote],
) -> Result<usize> {
    let mut loaded_count = 0;

    // Ensure attachments directory exists
    let attachments_dir = store.root().join("attachments");
    std::fs::create_dir_all(&attachments_dir)
        .map_err(|e| QipuError::Other(format!("failed to create attachments directory: {}", e)))?;

    for pack_attachment in pack_attachments {
        // Decode attachment data
        let data = general_purpose::STANDARD
            .decode(&pack_attachment.data)
            .map_err(|e| QipuError::Other(format!("failed to decode attachment data: {}", e)))?;

        // Validate path is within attachments directory (prevent path traversal)
        // We canonicalize the parent directory (which exists) and check the final path
        let canonical_attachments_dir = std::fs::canonicalize(&attachments_dir).map_err(|e| {
            QipuError::Other(format!("failed to resolve attachments directory: {}", e))
        })?;

        // Construct the expected canonical path by joining the canonical dir with the filename
        // This prevents path traversal attacks (e.g., "../secrets/file")
        let file_name = std::path::Path::new(&pack_attachment.name)
            .file_name()
            .ok_or_else(|| {
                QipuError::Other(format!(
                    "invalid attachment name '{}': no filename component",
                    pack_attachment.name
                ))
            })?;
        let safe_attachment_path = canonical_attachments_dir.join(file_name);

        // Write attachment to file system
        std::fs::write(&safe_attachment_path, data).map_err(|e| {
            QipuError::Other(format!(
                "failed to write attachment '{}': {}",
                pack_attachment.name, e
            ))
        })?;

        loaded_count += 1;
    }

    Ok(loaded_count)
}
