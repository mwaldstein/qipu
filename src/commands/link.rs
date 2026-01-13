//! Link management commands for qipu
//!
//! Per spec (specs/cli-interface.md, specs/graph-traversal.md):
//! - `qipu link list <id>` - list links for a note
//! - `qipu link add <from> <to> --type <t>` - add typed link
//! - `qipu link remove <from> <to> --type <t>` - remove typed link
//! - `qipu link tree <id>` - traversal tree from note
//! - `qipu link path <from> <to>` - find path between notes

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::index::{Edge, Index, IndexBuilder, LinkSource};
use crate::lib::note::{LinkType, TypedLink};
use crate::lib::store::Store;
use serde::Serialize;

/// Direction for link listing/traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Outbound links only (links FROM this note)
    Out,
    /// Inbound links only (backlinks TO this note)
    In,
    #[default]
    /// Both directions
    Both,
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "out" => Ok(Direction::Out),
            "in" => Ok(Direction::In),
            "both" => Ok(Direction::Both),
            other => Err(format!(
                "unknown direction '{}' (expected: out, in, both)",
                other
            )),
        }
    }
}

/// Link entry for output
#[derive(Debug, Clone, Serialize)]
pub struct LinkEntry {
    /// Direction relative to the queried note
    pub direction: String,
    /// The other note's ID
    pub id: String,
    /// The other note's title (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Link type (related, derived-from, supports, contradicts, part-of)
    #[serde(rename = "type")]
    pub link_type: String,
    /// Link source (typed or inline)
    pub source: String,
}

/// Execute the link list command
///
/// Lists all links for a note, with optional direction and type filters.
pub fn execute_list(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    direction: Direction,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Result<()> {
    // Resolve the note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build the index
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Verify note exists
    if !index.contains(&note_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound { id: note_id });
    }

    // Collect links based on direction
    let mut entries = Vec::new();

    // Outbound edges (links FROM this note)
    if direction == Direction::Out || direction == Direction::Both {
        for edge in index.get_outbound_edges(&note_id) {
            if let Some(entry) =
                filter_and_convert(edge, "out", &index, type_filter, typed_only, inline_only)
            {
                entries.push(entry);
            }
        }
    }

    // Inbound edges (backlinks TO this note)
    if direction == Direction::In || direction == Direction::Both {
        for edge in index.get_inbound_edges(&note_id) {
            if let Some(entry) =
                filter_and_convert_inbound(edge, &index, type_filter, typed_only, inline_only)
            {
                entries.push(entry);
            }
        }
    }

    // Sort for determinism: direction, then type, then id
    entries.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.id.cmp(&b.id))
    });

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Human => {
            if entries.is_empty() {
                if !cli.quiet {
                    println!("No links found for {}", note_id);
                }
            } else {
                for entry in &entries {
                    let dir_arrow = match entry.direction.as_str() {
                        "out" => "->",
                        "in" => "<-",
                        _ => "--",
                    };
                    let title_part = entry
                        .title
                        .as_ref()
                        .map(|t| format!(" \"{}\"", t))
                        .unwrap_or_default();
                    println!(
                        "{} {} {} [{}] ({})",
                        dir_arrow, entry.id, title_part, entry.link_type, entry.source
                    );
                }
            }
        }
        OutputFormat::Records => {
            // Header line
            println!(
                "H qipu=1 records=1 mode=link.list id={} direction={}",
                note_id,
                match direction {
                    Direction::Out => "out",
                    Direction::In => "in",
                    Direction::Both => "both",
                }
            );
            // Edge lines
            for entry in &entries {
                // E <from> <type> <to> <source>
                // For consistency, always show from -> to even for inbound
                let (from, to) = match entry.direction.as_str() {
                    "out" => (note_id.clone(), entry.id.clone()),
                    "in" => (entry.id.clone(), note_id.clone()),
                    _ => (note_id.clone(), entry.id.clone()),
                };
                println!("E {} {} {} {}", from, entry.link_type, to, entry.source);
            }
        }
    }

    Ok(())
}

/// Execute the link add command
///
/// Adds a typed link from one note to another.
pub fn execute_add(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    link_type: LinkType,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load and verify both notes exist
    let mut from_note = store.get_note(&from_resolved)?;
    let _to_note = store.get_note(&to_resolved)?;

    // Check if link already exists
    let link_exists = from_note
        .frontmatter
        .links
        .iter()
        .any(|l| l.id == to_resolved && l.link_type == link_type);

    if link_exists {
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "unchanged",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link already exists"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} already exists",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!("H qipu=1 records=1 mode=link.add status=unchanged");
                    println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
                }
            }
        }
        return Ok(());
    }

    // Add the link
    from_note.frontmatter.links.push(TypedLink {
        link_type,
        id: to_resolved.clone(),
    });

    // Save the note
    store.save_note(&from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "added",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Added link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=link.add status=added");
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}

/// Execute the link remove command
///
/// Removes a typed link from one note to another.
pub fn execute_remove(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    link_type: LinkType,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load the source note
    let mut from_note = store.get_note(&from_resolved)?;

    // Find and remove the link
    let original_len = from_note.frontmatter.links.len();
    from_note
        .frontmatter
        .links
        .retain(|l| !(l.id == to_resolved && l.link_type == link_type));

    if from_note.frontmatter.links.len() == original_len {
        // Link didn't exist
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "not_found",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link does not exist"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} does not exist",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!("H qipu=1 records=1 mode=link.remove status=not_found");
                }
            }
        }
        return Ok(());
    }

    // Save the note
    store.save_note(&from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "removed",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Removed link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=link.remove status=removed");
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}

/// Resolve a note ID or path to a canonical note ID
fn resolve_note_id(store: &Store, id_or_path: &str) -> Result<String> {
    // If it looks like an ID (starts with qp-), try to use it directly
    if id_or_path.starts_with("qp-") {
        // Could be a full filename like qp-xxxx-slug.md or just qp-xxxx
        let id = id_or_path
            .trim_end_matches(".md")
            .split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");
        return Ok(id);
    }

    // Otherwise, try to find a note by path
    let notes = store.list_notes()?;
    for note in notes {
        if let Some(path) = &note.path {
            let path_str = path.display().to_string();
            if path_str.contains(id_or_path) || path_str.ends_with(id_or_path) {
                return Ok(note.id().to_string());
            }
        }
    }

    Err(crate::lib::error::QipuError::NoteNotFound {
        id: id_or_path.to_string(),
    })
}

/// Filter and convert an outbound edge to a LinkEntry
fn filter_and_convert(
    edge: &Edge,
    direction: &str,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Option<LinkEntry> {
    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get target note title if available
    let title = index.get_metadata(&edge.to).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: direction.to_string(),
        id: edge.to.clone(),
        title,
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
    })
}

/// Filter and convert an inbound edge to a LinkEntry
fn filter_and_convert_inbound(
    edge: &Edge,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Option<LinkEntry> {
    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get source note title if available
    let title = index.get_metadata(&edge.from).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: "in".to_string(),
        id: edge.from.clone(),
        title,
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_parsing() {
        assert_eq!("out".parse::<Direction>().unwrap(), Direction::Out);
        assert_eq!("in".parse::<Direction>().unwrap(), Direction::In);
        assert_eq!("both".parse::<Direction>().unwrap(), Direction::Both);
        assert_eq!("OUT".parse::<Direction>().unwrap(), Direction::Out);
    }

    #[test]
    fn test_direction_parsing_invalid() {
        assert!("invalid".parse::<Direction>().is_err());
    }
}
