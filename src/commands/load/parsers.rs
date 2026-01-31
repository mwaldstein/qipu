use crate::commands::load::model::{PackAttachment, PackHeader, PackLink, PackSource};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::path::Path;

pub(super) fn parse_header_line(line: &str) -> PackHeader {
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

pub(super) fn parse_config_line(
    line: &str,
    config_encoded_lines: &mut Vec<String>,
    config_content: &mut String,
) {
    if let Some(stripped) = line.strip_prefix("CFG ") {
        config_encoded_lines.push(stripped.to_string());
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

pub(super) fn parse_source_line(line: &str) -> Option<PackSource> {
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

pub(super) fn parse_link_line(line: &str) -> Option<PackLink> {
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

pub(super) fn parse_attachment_metadata_line(line: &str) -> Option<PackAttachment> {
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

pub(super) fn handle_content_line(
    line: &str,
    content_lines: &mut Vec<String>,
    is_base64: &mut bool,
) {
    if let Some(stripped) = line.strip_prefix("B ") {
        *is_base64 = false;
        content_lines.push(stripped.to_string());
    } else if let Some(stripped) = line.strip_prefix("C ") {
        *is_base64 = true;
        content_lines.push(stripped.to_string());
    }
}
