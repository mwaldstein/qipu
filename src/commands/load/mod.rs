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
use crate::lib::error::{QipuError, Result};
use crate::lib::note::{Note, NoteFrontmatter, NoteType, Source};
use crate::lib::store::Store;
use model::{PackAttachment, PackLink, PackNote};

/// Execute the load command
pub fn execute(cli: &Cli, store: &Store, pack_file: &Path) -> Result<()> {
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

    // Load notes
    let loaded_notes_count = load_notes(store, &pack_data.notes)?;

    // Load links
    let loaded_links_count = load_links(store, &pack_data.links, &pack_data.notes)?;

    // Load attachments
    let loaded_attachments_count = if !pack_data.attachments.is_empty() {
        load_attachments(store, &pack_data.attachments, &pack_data.notes)?
    } else {
        0
    };

    // Report results
    if cli.verbose && !cli.quiet {
        eprintln!(
            "loaded {} notes, {} links, {} attachments from {}",
            loaded_notes_count,
            loaded_links_count,
            loaded_attachments_count,
            pack_file.display()
        );
    }

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
            println!(
                "H load=1 pack_file={} notes={} links={} attachments={}",
                pack_file.display(),
                loaded_notes_count,
                loaded_links_count,
                loaded_attachments_count
            );
        }
    }

    Ok(())
}

/// Load notes from pack into store
fn load_notes(store: &Store, pack_notes: &[PackNote]) -> Result<usize> {
    let mut loaded_count = 0;

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
            summary: None,
            compacts: Vec::new(),
        };

        // Create note
        let note = Note {
            frontmatter,
            body: pack_note.content.clone(),
            path: None, // Will be set by store when saving
        };

        // Save note to store
        let mut mutable_note = note;
        store.save_note(&mut mutable_note)?;
        loaded_count += 1;
    }

    Ok(loaded_count)
}

/// Load links from pack into store
fn load_links(store: &Store, pack_links: &[PackLink], loaded_notes: &[PackNote]) -> Result<usize> {
    let loaded_ids: HashSet<_> = loaded_notes.iter().map(|n| &n.id).collect();
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
        // Only process if the source note was loaded
        if loaded_ids.contains(&source_id) {
            // Load the source note
            let mut source_note = store.get_note(&source_id)?;

            // Add each link to the note's frontmatter
            for pack_link in links {
                // Only load links between notes that were loaded
                if loaded_ids.contains(&pack_link.to) {
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

            // Save the updated note
            store.save_note(&mut source_note)?;
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
