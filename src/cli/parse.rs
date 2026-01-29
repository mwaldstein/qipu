//! CLI argument parsers for clap
//!
//! Per custom ontology spec (specs/custom-ontology.md), validation of note and link types
//! is deferred to command execution where the active ontology is available. These parser
//! functions accept any string value and rely on clap's value_parser interface.
//!
//! Error messages showing valid types from the active ontology are generated in commands
//! when validation fails (e.g., capture/create/link commands).

use qipu_core::note::{LinkType, NoteType};

/// Parse note type from string
///
/// Validation is deferred to command execution where the active ontology is available.
/// This function accepts any string value; invalid types will be caught by commands
/// with error messages showing valid types from the active ontology.
pub fn parse_note_type(s: &str) -> std::result::Result<NoteType, String> {
    s.parse::<NoteType>().map_err(|e| e.to_string())
}

/// Parse link type from string
///
/// Validation is deferred to command execution where the active ontology is available.
/// This function accepts any string value; invalid types will be caught by commands
/// with error messages showing valid types from the active ontology.
pub fn parse_link_type(s: &str) -> std::result::Result<LinkType, String> {
    s.parse::<LinkType>().map_err(|e| e.to_string())
}

/// Parse minimum value (must be 0-100)
pub fn parse_min_value(s: &str) -> std::result::Result<u8, String> {
    let value: u8 = s.parse().map_err(|e| format!("Invalid value: {}", e))?;
    if value > 100 {
        Err("Value must be between 0 and 100".to_string())
    } else {
        Ok(value)
    }
}

/// Parse boolean from string (accepts true, false, 1, 0, yes, no)
pub fn parse_bool(s: &str) -> std::result::Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(format!(
            "Invalid boolean value: {}. Use true/false, 1/0, yes/no, or y/n",
            s
        )),
    }
}
