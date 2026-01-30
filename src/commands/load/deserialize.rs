#![allow(clippy::manual_strip)]

use super::model::{PackAttachment, PackData, PackHeader, PackLink, PackNote, PackSource};
use base64::{engine::general_purpose, Engine as _};
use qipu_core::error::{QipuError, Result};
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

fn parse_header_line(line: &str) -> PackHeader {
    let parts: Vec<_> = line[2..].split_whitespace().collect();
    let mut header_data = HashMap::new();

    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            header_data.insert(key, value);
        }
    }

    PackHeader {
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
        config_count: header_data
            .get("config")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    }
}

fn parse_config_line(
    line: &str,
    config_encoded_lines: &mut Vec<String>,
    config_content: &mut String,
) {
    if line.starts_with("CFG ") {
        config_encoded_lines.push(line[4..].to_string());
    } else if line == "CFG-END" && !config_encoded_lines.is_empty() {
        let encoded = config_encoded_lines.join("");
        if let Some(decoded_str) = general_purpose::STANDARD
            .decode(&encoded)
            .ok()
            .and_then(|decoded| String::from_utf8(decoded).ok())
        {
            *config_content = decoded_str;
        }
        config_encoded_lines.clear();
    }
}

fn parse_note_metadata_line(line: &str) -> Option<PackNote> {
    let mut parts = line[2..].splitn(3, ' ');
    let id = parts.next().unwrap_or("").to_string();
    let note_type = parts.next().unwrap_or("").to_string();
    let remainder = parts.next().unwrap_or("").trim();

    if id.is_empty() || note_type.is_empty() {
        return None;
    }

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

    let metadata = parse_note_metadata(metadata_str);

    Some(PackNote {
        id,
        title,
        note_type,
        tags: metadata.tags,
        created: metadata.created,
        updated: metadata.updated,
        path: None,
        content: String::new(),
        sources: Vec::new(),
        summary: metadata.summary,
        compacts: metadata.compacts,
        source: metadata.source,
        author: metadata.author,
        generated_by: metadata.generated_by,
        prompt_hash: metadata.prompt_hash,
        verified: metadata.verified,
        value: metadata.value,
        custom: metadata.custom,
    })
}

struct NoteMetadata {
    tags: Vec<String>,
    created: Option<chrono::DateTime<chrono::Utc>>,
    updated: Option<chrono::DateTime<chrono::Utc>>,
    summary: Option<String>,
    compacts: Vec<String>,
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
    value: Option<u8>,
    custom: HashMap<String, serde_json::Value>,
}

fn parse_value(metadata_chars: &[char], current_pos: &mut usize) -> String {
    if *current_pos < metadata_chars.len() && metadata_chars[*current_pos] == '"' {
        *current_pos += 1;
        let quote_start = *current_pos;
        while *current_pos < metadata_chars.len() && metadata_chars[*current_pos] != '"' {
            *current_pos += 1;
        }
        let val = metadata_chars[quote_start..*current_pos].iter().collect();
        if *current_pos < metadata_chars.len() {
            *current_pos += 1;
        }
        val
    } else {
        let val_start = *current_pos;
        while *current_pos < metadata_chars.len() && !metadata_chars[*current_pos].is_whitespace() {
            *current_pos += 1;
        }
        metadata_chars[val_start..*current_pos].iter().collect()
    }
}

fn apply_metadata_value(key: &str, val: &str, metadata: &mut NoteMetadata) {
    match key {
        "tags" => {
            if val != "-" {
                metadata.tags = val.split(',').map(|s| s.to_string()).collect();
            }
        }
        "created" => metadata.created = val.parse().ok(),
        "updated" => metadata.updated = val.parse().ok(),
        "summary" => metadata.summary = Some(val.to_string()),
        "compacts" => {
            if val != "-" {
                metadata.compacts = val.split(',').map(|s| s.to_string()).collect();
            }
        }
        "source" => metadata.source = Some(val.to_string()),
        "author" => metadata.author = Some(val.to_string()),
        "generated_by" => metadata.generated_by = Some(val.to_string()),
        "prompt_hash" => metadata.prompt_hash = Some(val.to_string()),
        "verified" => metadata.verified = val.parse().ok(),
        "value" => metadata.value = val.parse().ok(),
        "custom" => {
            if let Ok(decoded) = general_purpose::STANDARD.decode(val) {
                if let Ok(json_str) = String::from_utf8(decoded) {
                    if let Ok(parsed) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&json_str)
                    {
                        metadata.custom = parsed;
                    }
                }
            }
        }
        _ => {
            // Unknown metadata keys are intentionally ignored for forward compatibility
        }
    }
}

fn parse_key_value_pairs(metadata_str: &str) -> NoteMetadata {
    let mut metadata = NoteMetadata {
        tags: Vec::new(),
        created: None,
        updated: None,
        summary: None,
        compacts: Vec::new(),
        source: None,
        author: None,
        generated_by: None,
        prompt_hash: None,
        verified: None,
        value: None,
        custom: HashMap::new(),
    };

    let mut current_pos = 0;
    let metadata_chars: Vec<char> = metadata_str.chars().collect();

    while current_pos < metadata_chars.len() {
        while current_pos < metadata_chars.len() && metadata_chars[current_pos].is_whitespace() {
            current_pos += 1;
        }
        if current_pos >= metadata_chars.len() {
            break;
        }

        let key_start = current_pos;
        while current_pos < metadata_chars.len()
            && metadata_chars[current_pos] != '='
            && !metadata_chars[current_pos].is_whitespace()
        {
            current_pos += 1;
        }
        let key: String = metadata_chars[key_start..current_pos].iter().collect();

        if current_pos < metadata_chars.len() && metadata_chars[current_pos] == '=' {
            current_pos += 1;
            let val = parse_value(&metadata_chars, &mut current_pos);
            apply_metadata_value(&key, &val, &mut metadata);
        } else {
            current_pos += 1;
        }
    }

    metadata
}

fn parse_note_metadata(metadata_str: &str) -> NoteMetadata {
    parse_key_value_pairs(metadata_str)
}

fn parse_source_line(line: &str) -> Option<PackSource> {
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

    if url.is_empty() {
        return None;
    }

    Some(PackSource {
        url,
        title,
        accessed,
    })
}

fn parse_link_line(line: &str) -> Option<PackLink> {
    let parts: Vec<_> = line[2..].split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

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

    Some(PackLink {
        from,
        to,
        link_type,
        inline,
    })
}

fn parse_attachment_metadata_line(line: &str) -> Option<PackAttachment> {
    let parts: Vec<_> = line[2..].split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

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

    Some(PackAttachment {
        path,
        name,
        data: String::new(),
        content_type,
    })
}

fn handle_content_line(line: &str, content_lines: &mut Vec<String>, is_base64: &mut bool) {
    if line.starts_with("B ") {
        *is_base64 = false;
        content_lines.push(line[2..].to_string());
    } else if line.starts_with("C ") {
        *is_base64 = true;
        content_lines.push(line[2..].to_string());
    }
}

/// Parse pack in records format
pub fn parse_records_pack(content: &str) -> Result<PackData> {
    let mut header: Option<PackHeader> = None;
    let mut notes: Vec<PackNote> = Vec::new();
    let mut links: Vec<PackLink> = Vec::new();
    let mut attachments: Vec<PackAttachment> = Vec::new();
    let mut config_content: String = String::new();
    let mut config_encoded_lines: Vec<String> = Vec::new();

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
            header = Some(parse_header_line(line));
        } else if line.starts_with("CFG ") || line == "CFG-END" {
            parse_config_line(line, &mut config_encoded_lines, &mut config_content);
        } else if line.starts_with("N ") {
            if let Some(ref mut note) = current_note {
                finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
                notes.push(note.clone());
            }
            current_note = parse_note_metadata_line(line);
        } else if line.starts_with("B ") || line.starts_with("C ") {
            handle_content_line(line, &mut note_content_lines, &mut note_content_is_base64);
        } else if line.starts_with("S ") {
            if let Some(ref mut note) = current_note {
                if let Some(source) = parse_source_line(line) {
                    note.sources.push(source);
                }
            }
        } else if line.starts_with("L ") {
            if let Some(link) = parse_link_line(line) {
                links.push(link);
            }
        } else if line.starts_with("A ") {
            if let Some(ref mut attachment) = current_attachment {
                attachments.push(attachment.clone());
            }
            current_attachment = parse_attachment_metadata_line(line);
        } else if line.starts_with("D ") {
            if let Some(ref mut attachment) = current_attachment {
                attachment.data.push_str(&line[2..]);
            }
        } else if line == "C-END" {
            if let Some(ref mut note) = current_note {
                finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
            }
        } else if line == "D-END" || line == "A-END" {
            if let Some(attachment) = current_attachment.take() {
                attachments.push(attachment);
            }
        } else if line == "END" {
            break;
        }
    }

    if let Some(ref mut note) = current_note {
        finalize_note_content(note, &mut note_content_lines, &mut note_content_is_base64)?;
        notes.push(note.clone());
    }

    if let Some(attachment) = current_attachment {
        attachments.push(attachment);
    }

    let header =
        header.ok_or_else(|| QipuError::Other("missing header in pack file".to_string()))?;

    Ok(PackData {
        header,
        notes,
        links,
        attachments,
        config_content,
    })
}
