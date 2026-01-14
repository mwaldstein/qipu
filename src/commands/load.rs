//! `qipu load` command - load notes from a pack file
//!
//! Per spec (specs/pack.md):
//! - Load pack file into store
//! - Restore notes, links, and attachments
//! - No content transformation
//! - Handle merge semantics for loading into non-empty stores

use std::collections::HashMap;
use std::path::Path;

use base64::{engine::general_purpose, Engine as _};

use crate::cli::Cli;
use crate::lib::error::{QipuError, Result};
use crate::lib::note::{Note, NoteFrontmatter, NoteType, Source};
use crate::lib::store::Store;

/// Pack file header
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PackHeader {
    version: String,
    created: chrono::DateTime<chrono::Utc>,
    store_path: String,
    notes_count: usize,
    attachments_count: usize,
    links_count: usize,
}

/// Pack entry for a note
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct PackNote {
    id: String,
    title: String,
    note_type: String,
    tags: Vec<String>,
    created: Option<chrono::DateTime<chrono::Utc>>,
    updated: Option<chrono::DateTime<chrono::Utc>>,
    path: Option<String>,
    content: String,
    sources: Vec<PackSource>,
}

/// Pack entry for a source reference
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct PackSource {
    url: String,
    title: Option<String>,
    accessed: Option<String>,
}

/// Pack entry for a link
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct PackLink {
    from: String,
    to: String,
    link_type: Option<String>,
    inline: bool,
}

/// Pack entry for an attachment
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct PackAttachment {
    path: String,
    name: String,
    data: String, // Base64 encoded
    content_type: Option<String>,
}

/// Complete pack data structure
#[derive(Debug, serde::Deserialize)]
struct PackData {
    header: PackHeader,
    notes: Vec<PackNote>,
    links: Vec<PackLink>,
    attachments: Vec<PackAttachment>,
}

/// Execute the load command
pub fn execute(cli: &Cli, store: &Store, pack_file: &Path) -> Result<()> {
    use crate::cli::OutputFormat;

    // Read pack file
    let pack_content = std::fs::read_to_string(pack_file)
        .map_err(|e| QipuError::Other(format!("failed to read pack file: {}", e)))?;

    // Parse pack content based on format
    let pack_data = if looks_like_json(&pack_content) {
        parse_json_pack(&pack_content)?
    } else {
        parse_records_pack(&pack_content)?
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
    if !cli.quiet {
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

/// Check if content looks like JSON (starts with '{')
fn looks_like_json(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with('{')
}

/// Parse pack in JSON format
fn parse_json_pack(content: &str) -> Result<PackData> {
    serde_json::from_str(content)
        .map_err(|e| QipuError::Other(format!("failed to parse JSON pack: {}", e)))
}

/// Parse pack in records format
fn parse_records_pack(content: &str) -> Result<PackData> {
    let mut header: Option<PackHeader> = None;
    let mut notes: Vec<PackNote> = Vec::new();
    let mut links: Vec<PackLink> = Vec::new();
    let mut attachments: Vec<PackAttachment> = Vec::new();

    let mut current_note: Option<PackNote> = None;
    let mut current_attachment: Option<PackAttachment> = None;
    let mut note_content_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("H ") {
            // Header line
            let parts: Vec<_> = line[2..].split_whitespace().collect();
            let mut header_data = HashMap::new();

            for part in parts {
                if let Some((key, value)) = part.split_once('=') {
                    header_data.insert(key, value);
                }
            }

            header = Some(PackHeader {
                version: header_data.get("version").unwrap_or(&"1.0").to_string(),
                created: header_data
                    .get("created")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| chrono::Utc::now()),
                store_path: header_data.get("store").unwrap_or(&"").to_string(),
                notes_count: header_data
                    .get("notes")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                attachments_count: header_data
                    .get("attachments")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                links_count: header_data
                    .get("links")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            });
        } else if line.starts_with("N ") {
            // Note metadata line
            if let Some(ref mut note) = current_note {
                // Finalize previous note
                note.content = note_content_lines.join("\n");
                notes.push(note.clone());
                note_content_lines.clear();
            }

            let parts: Vec<_> = line[2..].split_whitespace().collect();
            if parts.len() >= 3 {
                let id = parts[0].to_string();
                let note_type = parts[1].to_string();
                let title = parts[2].trim_matches('"').to_string();

                // Parse tags from remaining parts
                let mut tags = Vec::new();
                let mut created = None;
                let mut updated = None;

                for part in parts.iter().skip(3) {
                    if let Some(tags_str) = part.strip_prefix("tags=") {
                        if tags_str != "-" {
                            tags = tags_str.split(',').map(|s| s.to_string()).collect();
                        }
                    } else if let Some(created_str) = part.strip_prefix("created=") {
                        created = created_str.parse().ok();
                    } else if let Some(updated_str) = part.strip_prefix("updated=") {
                        updated = updated_str.parse().ok();
                    }
                }

                current_note = Some(PackNote {
                    id,
                    title,
                    note_type,
                    tags,
                    created,
                    updated,
                    path: None,
                    content: String::new(),
                    sources: Vec::new(),
                });
            }
        } else if line.starts_with("B ") {
            // Body content line - collect it
            let content_line = line[2..].to_string();
            note_content_lines.push(content_line);
        } else if line.starts_with("S ") {
            // Source line
            if let Some(ref mut note) = current_note {
                let parts: Vec<_> = line[2..].split_whitespace().collect();
                let mut url = String::new();
                let mut title = None;
                let mut accessed = None;

                for part in parts {
                    if let Some(url_str) = part.strip_prefix("url=") {
                        url = url_str.to_string();
                    } else if let Some(title_str) = part.strip_prefix("title=") {
                        title = Some(title_str.trim_matches('"').to_string());
                    } else if let Some(accessed_str) = part.strip_prefix("accessed=") {
                        accessed = Some(accessed_str.to_string());
                    }
                }

                if !url.is_empty() {
                    note.sources.push(PackSource {
                        url,
                        title,
                        accessed,
                    });
                }
            }
        } else if line.starts_with("L ") {
            // Link line
            let parts: Vec<_> = line[2..].split_whitespace().collect();
            if parts.len() >= 2 {
                let from = parts[0].to_string();
                let to = parts[1].to_string();
                let link_type = parts
                    .get(2)
                    .and_then(|s| s.strip_prefix("type="))
                    .map(|s| s.to_string());
                let inline = parts
                    .get(3)
                    .map(|s| s.contains("inline=true"))
                    .unwrap_or(false);

                links.push(PackLink {
                    from,
                    to,
                    link_type,
                    inline,
                });
            }
        } else if line.starts_with("A ") {
            // Attachment line
            if let Some(ref mut attachment) = current_attachment {
                // Finalize previous attachment if any
                attachments.push(attachment.clone());
            }

            let parts: Vec<_> = line[2..].split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[0].to_string();
                let name = parts[1].to_string();
                let content_type = parts
                    .get(2)
                    .and_then(|s| s.strip_prefix("content_type="))
                    .map(|s| s.to_string());

                current_attachment = Some(PackAttachment {
                    path,
                    name,
                    data: String::new(), // Will be filled by D lines
                    content_type,
                });
            }
        } else if line.starts_with("D ") {
            // Attachment data line
            if let Some(ref mut attachment) = current_attachment {
                attachment.data.push_str(&line[2..]);
            }
        } else if line == "C-END" {
            // End of note/content block
            if let Some(ref mut note) = current_note {
                note.content = note_content_lines.join("\n");
                notes.push(note.clone());
                note_content_lines.clear();
            }
            current_note = None;
        } else if line == "A-END" {
            // End of attachment block
            if let Some(attachment) = current_attachment.take() {
                attachments.push(attachment);
            }
        } else if line == "END" {
            // End of pack
            break;
        }
    }

    // Flush any pending note
    if let Some(ref mut note) = current_note {
        note.content = note_content_lines.join("\n");
        notes.push(note.clone());
    }

    // Flush any pending attachment
    if let Some(attachment) = current_attachment {
        attachments.push(attachment);
    }

    // Validate we have a header
    let header =
        header.ok_or_else(|| QipuError::Other("missing header in pack file".to_string()))?;

    Ok(PackData {
        header,
        notes,
        links,
        attachments,
    })
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
    let loaded_ids: std::collections::HashSet<_> = loaded_notes.iter().map(|n| &n.id).collect();
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
