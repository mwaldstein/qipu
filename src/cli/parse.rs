use crate::lib::note::{LinkType, NoteType};

/// Parse note type from string
pub fn parse_note_type(s: &str) -> std::result::Result<NoteType, String> {
    s.parse::<NoteType>().map_err(|e| e.to_string())
}

/// Parse link type from string
pub fn parse_link_type(s: &str) -> std::result::Result<LinkType, String> {
    s.parse::<LinkType>().map_err(|e| e.to_string())
}
