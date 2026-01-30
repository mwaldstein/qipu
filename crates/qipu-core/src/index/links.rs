#![allow(clippy::unnecessary_unwrap)]

use super::types::{Edge, LinkSource};
use crate::error::Result;
use crate::note::Note;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Extract all links from a note
#[tracing::instrument(skip(note, valid_ids, unresolved, source_path, path_to_id), fields(note_id = %note.id(), source_path = ?source_path))]
pub(crate) fn extract_links(
    note: &Note,
    valid_ids: &HashSet<String>,
    unresolved: &mut HashSet<String>,
    source_path: Option<&Path>,
    path_to_id: &HashMap<PathBuf, String>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    // Extract typed links from frontmatter
    for link in &note.frontmatter.links {
        let to_id = link.id.clone();
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id);
            continue;
        }
        edges.push(Edge {
            from: note.id().to_string(),
            to: to_id,
            link_type: link.link_type.clone(),
            source: LinkSource::Typed,
        });
    }

    // Extract wiki links from body: [[id]] or [[id|label]]
    let wiki_link_re = match Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]") {
        Ok(re) => re,
        Err(e) => {
            warn!(error = %e, "Failed to compile wiki link regex");
            return edges; // Return empty edges if regex fails
        }
    };
    for cap in wiki_link_re.captures_iter(&note.body) {
        let to_id = cap[1].trim().to_string();
        if to_id.is_empty() {
            continue;
        }
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id);
            continue;
        }
        // Inline links default to "related" type
        edges.push(Edge {
            from: note.id().to_string(),
            to: to_id,
            link_type: crate::note::LinkType::from("related"),
            source: LinkSource::Inline,
        });
    }

    // Extract markdown links to qipu notes: [text](qp-xxxx) or [text](./qp-xxxx-slug.md) or [text](relative/path.md)
    let md_link_re = match Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(e) => {
            warn!(error = %e, "Failed to compile markdown link regex");
            return edges; // Return empty edges if regex fails
        }
    };
    for cap in md_link_re.captures_iter(&note.body) {
        let target = cap[2].trim();

        // Skip external URLs and anchors
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('#')
        {
            continue;
        }

        // Try to resolve the link to a note ID
        let to_id = if target.starts_with("qp-") {
            // Direct ID reference: [text](qp-xxxx)
            Some(target.split('-').take(2).collect::<Vec<_>>().join("-"))
        } else if target.contains("qp-") {
            // Path reference containing ID: [text](./qp-xxxx-slug.md)
            if let Some(start) = target.find("qp-") {
                let rest = &target[start..];
                // Extract the ID portion (qp-xxxx)
                let end = rest
                    .find('-')
                    .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
                match end {
                    Some(end) => Some(rest[..end].to_string()),
                    None => Some(rest.trim_end_matches(".md").to_string()),
                }
            } else {
                None
            }
        } else if target.ends_with(".md") && source_path.is_some() {
            // Relative path to markdown file: [text](../other/note.md)
            let source = source_path.unwrap();
            if let Some(source_dir) = source.parent() {
                let target_path = source_dir.join(target);

                // Canonicalize the path to resolve .. and .
                let canonical_target = match target_path.canonicalize() {
                    Ok(p) => p,
                    Err(_) => {
                        // Path doesn't exist, try without canonicalizing
                        // (might be a reference to a note that will be created)
                        target_path
                    }
                };

                // Look up the ID from the path
                path_to_id.get(&canonical_target).cloned()
            } else {
                None
            }
        } else {
            None
        };

        let to_id = match to_id {
            Some(id) if !id.is_empty() && id.starts_with("qp-") => id,
            _ => continue,
        };

        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id);
            continue;
        }

        edges.push(Edge {
            from: note.id().to_string(),
            to: to_id,
            link_type: crate::note::LinkType::from("related"),
            source: LinkSource::Inline,
        });
    }

    // Deduplicate edges by (to, link_type) only, keeping the first occurrence
    // This ensures typed links from frontmatter take precedence over inline links
    // when both exist to the same target with the same type
    // NOTE: We do NOT sort here to preserve order from frontmatter
    edges.dedup_by(|a, b| a.to == b.to && a.link_type == b.link_type);

    edges
}

/// Rewrite wiki-links \[\[id\]\] and \[\[id|label\]\] to markdown links [label](qp-id.md)
pub fn rewrite_wiki_links(note: &mut Note) -> Result<bool> {
    let wiki_link_re = match Regex::new(r"\[\[([^\]]+)\]\]") {
        Ok(re) => re,
        Err(e) => {
            warn!(error = %e, "Failed to compile wiki link regex");
            return Ok(false);
        }
    };

    let mut modified = false;
    let mut last_end = 0;
    let mut new_body = String::new();

    for cap in wiki_link_re.captures_iter(&note.body) {
        let content = cap[1].trim();
        if content.is_empty() {
            continue;
        }

        let (id, label) = if content.contains('|') {
            let parts: Vec<&str> = content.splitn(2, '|').collect();
            (parts[0].trim().to_string(), parts[1].trim().to_string())
        } else {
            (content.to_string(), content.to_string())
        };

        if id.is_empty() {
            continue;
        }

        let md_link = format!("[{}]({}.md)", label, id);

        let full_match = cap.get(0).unwrap();
        new_body.push_str(&note.body[last_end..full_match.start()]);
        new_body.push_str(&md_link);
        last_end = full_match.end();
        modified = true;
    }

    if modified {
        new_body.push_str(&note.body[last_end..]);
        note.body = new_body;
    }

    Ok(modified)
}
