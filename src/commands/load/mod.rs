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
use std::sync::atomic::{AtomicBool, Ordering};

use base64::{engine::general_purpose, Engine as _};

use crate::cli::{Cli, OutputFormat};
use crate::lib::config::STORE_FORMAT_VERSION;
use crate::lib::error::{QipuError, Result};
use crate::lib::note::{Note, NoteFrontmatter, NoteType, Source, TypedLink};
use crate::lib::store::Store;
use model::{PackAttachment, PackLink, PackNote};

static VERBOSE_ENABLED: AtomicBool = AtomicBool::new(false);

fn verbose_enabled() -> bool {
    VERBOSE_ENABLED.load(Ordering::Relaxed)
}

fn set_verbose(enabled: bool) {
    VERBOSE_ENABLED.store(enabled, Ordering::Relaxed);
}

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
    // Set verbose flag for use in conflict resolution
    set_verbose(cli.verbose);

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
    let loaded_notes_count = load_notes(store, &pack_data.notes, &strategy)?;

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

fn write_note_preserving_updated(note: &Note) -> Result<()> {
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
    }

    Ok(())
}

/// Load notes from pack into store
fn load_notes(store: &Store, pack_notes: &[PackNote], strategy: &LoadStrategy) -> Result<usize> {
    let mut loaded_count = 0;
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
        };

        // Create note
        let mut note = Note {
            frontmatter,
            body: pack_note.content.clone(),
            path: None,
        };

        // Determine target directory and filename
        let target_dir = match note_type {
            NoteType::Moc => store.mocs_dir(),
            _ => store.notes_dir(),
        };

        // Use a deterministic filename based on ID and title
        let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
        let file_path = target_dir.join(&file_name);
        note.path = Some(file_path);

        // Handle conflicts based on strategy
        let should_load = if existing_ids.contains(note.id()) {
            match strategy {
                LoadStrategy::Skip => {
                    if verbose_enabled() {
                        eprintln!(
                            "Skipping conflicting note: {} (ID: {})",
                            note.title(),
                            note.id()
                        );
                    }
                    false
                }
                LoadStrategy::Overwrite => {
                    if verbose_enabled() {
                        eprintln!(
                            "Overwriting existing note: {} (ID: {})",
                            note.title(),
                            note.id()
                        );
                    }
                    true
                }
                LoadStrategy::MergeLinks => {
                    // Merge links from existing note into pack note
                    if let Ok(existing_note) = store.get_note(&note.id()) {
                        // Union of links, deduplicating by (id, link_type)
                        let existing_links = &existing_note.frontmatter.links;
                        let pack_links: Vec<TypedLink> = note
                            .frontmatter
                            .links
                            .iter()
                            .filter(|l| {
                                !existing_links
                                    .iter()
                                    .any(|el| el.id == l.id && el.link_type == l.link_type)
                            })
                            .cloned()
                            .collect();
                        note.frontmatter.links.extend(pack_links);

                        if verbose_enabled() {
                            eprintln!(
                                "Merging links for note: {} (ID: {})",
                                note.title(),
                                note.id()
                            );
                        }
                    }
                    true
                }
            }
        } else {
            true
        };

        if should_load {
            // Save note without overwriting pack timestamps
            write_note_preserving_updated(&note)?;
            loaded_count += 1;
        }
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

            write_note_preserving_updated(&source_note)?;
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
