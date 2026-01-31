#![allow(clippy::manual_strip)]

use super::model::{PackAttachment, PackData, PackHeader, PackLink, PackNote};
use base64::{engine::general_purpose, Engine as _};
use qipu_core::error::{QipuError, Result};

use crate::commands::load::{
    metadata::parse_note_metadata_line,
    parsers::{
        handle_content_line, parse_attachment_metadata_line, parse_config_line, parse_header_line,
        parse_link_line, parse_source_line,
    },
};

pub fn looks_like_json(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with('{')
}

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
