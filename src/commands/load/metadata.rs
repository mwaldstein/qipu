use crate::commands::load::model::PackNote;
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;

pub(super) fn parse_note_metadata_line(line: &str) -> Option<PackNote> {
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
        _ => {}
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
