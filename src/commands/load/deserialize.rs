#![allow(clippy::manual_strip)]

use super::model::{PackAttachment, PackData, PackHeader, PackLink, PackNote, PackSource};
use crate::lib::error::{QipuError, Result};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::path::Path;

/// Check if content looks like JSON (starts with '{')
pub fn looks_like_json(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with('{')
}

/// Parse pack in JSON format
pub fn parse_json_pack(content: &str) -> Result<PackData> {
    serde_json::from_str(content)
        .map_err(|e| QipuError::Other(format!("failed to parse JSON pack: {}", e)))
}

fn finalize_note_content(
    note: &mut PackNote,
    note_content_lines: &mut Vec<String>,
    note_content_is_base64: &mut bool,
) -> Result<()> {
    if note_content_lines.is_empty() {
        *note_content_is_base64 = false;
        return Ok(());
    }

    let content = if *note_content_is_base64 {
        let encoded = note_content_lines.join("");
        let decoded = general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| QipuError::Other(format!("failed to decode note content: {}", e)))?;
        String::from_utf8(decoded)
            .map_err(|e| QipuError::Other(format!("failed to decode note content: {}", e)))?
    } else {
        note_content_lines.join("\n")
    };

    note.content = content;
    note_content_lines.clear();
    *note_content_is_base64 = false;
    Ok(())
}

/// Parse pack in records format
pub fn parse_records_pack(content: &str) -> Result<PackData> {
    let mut header: Option<PackHeader> = None;
    let mut notes: Vec<PackNote> = Vec::new();
    let mut links: Vec<PackLink> = Vec::new();
    let mut attachments: Vec<PackAttachment> = Vec::new();

    let mut current_note: Option<PackNote> = None;
    let mut current_attachment: Option<PackAttachment> = None;
    let mut note_content_lines: Vec<String> = Vec::new();
    let mut note_content_is_base64 = false;

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
                store_version: header_data
                    .get("store_version")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                created: header_data
                    .get("created")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(chrono::Utc::now),
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
                finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
                notes.push(note.clone());
            }

            let mut parts = line[2..].splitn(3, ' ');
            let id = parts.next().unwrap_or("").to_string();
            let note_type = parts.next().unwrap_or("").to_string();
            let remainder = parts.next().unwrap_or("").trim();

            if !id.is_empty() && !note_type.is_empty() {
                let (title, metadata_str) = if let Some(stripped) = remainder.strip_prefix('"') {
                    if let Some(end_quote) = stripped.find('"') {
                        let title = stripped[..end_quote].to_string();
                        let rest = stripped[end_quote + 1..].trim();
                        (title, rest)
                    } else {
                        (stripped.to_string(), "")
                    }
                } else {
                    let mut title_parts = remainder.splitn(2, ' ');
                    let title = title_parts.next().unwrap_or("").to_string();
                    let rest = title_parts.next().unwrap_or("").trim();
                    (title, rest)
                };

                // Parse metadata
                let mut tags = Vec::new();
                let mut created = None;
                let mut updated = None;
                let mut summary = None;
                let mut compacts = Vec::new();
                let mut source = None;
                let mut author = None;
                let mut generated_by = None;
                let mut prompt_hash = None;
                let mut verified = None;

                // Split metadata_str carefully to handle quoted values
                let mut current_pos = 0;
                let metadata_chars: Vec<char> = metadata_str.chars().collect();

                while current_pos < metadata_chars.len() {
                    // Skip whitespace
                    while current_pos < metadata_chars.len()
                        && metadata_chars[current_pos].is_whitespace()
                    {
                        current_pos += 1;
                    }
                    if current_pos >= metadata_chars.len() {
                        break;
                    }

                    // Find key
                    let key_start = current_pos;
                    while current_pos < metadata_chars.len()
                        && metadata_chars[current_pos] != '='
                        && !metadata_chars[current_pos].is_whitespace()
                    {
                        current_pos += 1;
                    }
                    let key: String = metadata_chars[key_start..current_pos].iter().collect();

                    if current_pos < metadata_chars.len() && metadata_chars[current_pos] == '=' {
                        current_pos += 1; // skip '='
                        let val_start = current_pos;
                        let val: String;

                        if current_pos < metadata_chars.len() && metadata_chars[current_pos] == '"'
                        {
                            current_pos += 1; // skip opening quote
                            let quote_start = current_pos;
                            while current_pos < metadata_chars.len()
                                && metadata_chars[current_pos] != '"'
                            {
                                current_pos += 1;
                            }
                            val = metadata_chars[quote_start..current_pos].iter().collect();
                            if current_pos < metadata_chars.len() {
                                current_pos += 1;
                            } // skip closing quote
                        } else {
                            while current_pos < metadata_chars.len()
                                && !metadata_chars[current_pos].is_whitespace()
                            {
                                current_pos += 1;
                            }
                            val = metadata_chars[val_start..current_pos].iter().collect();
                        }

                        match key.as_str() {
                            "tags" => {
                                if val != "-" {
                                    tags = val.split(',').map(|s| s.to_string()).collect();
                                }
                            }
                            "created" => created = val.parse().ok(),
                            "updated" => updated = val.parse().ok(),
                            "summary" => summary = Some(val),
                            "compacts" => {
                                if val != "-" {
                                    compacts = val.split(',').map(|s| s.to_string()).collect();
                                }
                            }
                            "source" => source = Some(val),
                            "author" => author = Some(val),
                            "generated_by" => generated_by = Some(val),
                            "prompt_hash" => prompt_hash = Some(val),
                            "verified" => verified = val.parse().ok(),
                            _ => {}
                        }
                    } else {
                        current_pos += 1;
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
                    summary,
                    compacts,
                    source,
                    author,
                    generated_by,
                    prompt_hash,
                    verified,
                });
            }
        } else if line.starts_with("B ") {
            // Body content line - collect it
            note_content_is_base64 = false;
            let content_line = line[2..].to_string();
            note_content_lines.push(content_line);
        } else if line.starts_with("C ") {
            // Base64-encoded content line
            note_content_is_base64 = true;
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
            if !parts.is_empty() {
                let path = parts[0].to_string();
                let mut name = None;
                let mut content_type = None;

                for part in parts.iter().skip(1) {
                    if let Some(name_str) = part.strip_prefix("name=") {
                        name = Some(name_str.to_string());
                    } else if let Some(content_type_str) = part.strip_prefix("content_type=") {
                        if content_type_str != "-" {
                            content_type = Some(content_type_str.to_string());
                        }
                    }
                }

                let name = name.unwrap_or_else(|| {
                    Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&path)
                        .to_string()
                });

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
                finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
            }
        } else if line == "D-END" || line == "A-END" {
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
        finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
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
