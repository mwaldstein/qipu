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
pub fn execute(cli: &Cli, store: &Store, pack_file: &Path, strategy: &str) -> Result<()> {
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

    // Load notes
    let (loaded_notes_count, loaded_ids, new_ids) = load_notes(store, &pack_data.notes, &strategy)?;

    // Load links
    // Get current existing IDs (including newly loaded notes) for edge resolution
    let all_existing_ids = store.existing_ids()?;
    // With skip strategy, only load links between newly loaded notes
    let loaded_links_count = if matches!(strategy, LoadStrategy::Skip) {
        load_links(store, &pack_data.links, &new_ids, &all_existing_ids)?
    } else {
        load_links(store, &pack_data.links, &loaded_ids, &all_existing_ids)?
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

fn write_note_preserving_updated(store: &Store, note: &Note, existing_ids: &HashSet<String>) -> Result<()> {
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
            note_type: Some(note_type),
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
        };

        // Create note
        let mut note = Note {
            frontmatter,
            body: pack_note.content.clone(),
            path: None,
        };

        // Determine target directory
        let target_dir = match note_type {
            NoteType::Moc => store.mocs_dir(),
            _ => store.notes_dir(),
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
            // New note - determine filename
            let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
            note.path = Some(target_dir.join(&file_name));
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
fn load_links(
    store: &Store,
    pack_links: &[PackLink],
    loaded_ids: &HashSet<String>,
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
        // Only process links for notes that were actually loaded
        if loaded_ids.contains(&source_id) {
            // Load the source note
            let mut source_note = store.get_note(&source_id)?;

            // Add each link to the note's frontmatter
            for pack_link in links {
                // Only load links where target note was loaded
                // This prevents adding links to skipped notes
                if loaded_ids.contains(&pack_link.to) {
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

        // Determine attachment path
        let attachment_path = attachments_dir.join(&pack_attachment.name);

        // Write attachment to file system
        std::fs::write(&attachment_path, data).map_err(|e| {
            QipuError::Other(format!(
                "failed to write attachment '{}': {}",
                pack_attachment.name, e
            ))
        })?;

        loaded_count += 1;
    }

    Ok(loaded_count)
}
