#![allow(clippy::unnecessary_unwrap)]

use super::types::{Edge, LinkSource};
use crate::lib::note::Note;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Extract all links from a note
pub(crate) fn extract_links(
    note: &Note,
    valid_ids: &HashSet<String>,
    unresolved: &mut HashSet<String>,
    source_path: Option<&Path>,
    path_to_id: &HashMap<PathBuf, String>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let from_id = note.id().to_string();

    // Extract typed links from frontmatter
    for link in &note.frontmatter.links {
        let to_id = link.id.clone();
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
            continue;
        }
        edges.push(Edge {
            from: from_id.clone(),
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
            unresolved.insert(to_id.clone());
            continue;
        }
        // Inline links default to "related" type
        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: crate::lib::note::LinkType::from("related"),
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
            unresolved.insert(to_id.clone());
            continue;
        }

        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: crate::lib::note::LinkType::from("related"),
            source: LinkSource::Inline,
        });
    }

    // Deduplicate edges (same from, to, type, source)
    edges.sort_by(|a, b| {
        a.to.cmp(&b.to)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| format!("{:?}", a.source).cmp(&format!("{:?}", b.source)))
    });
    edges.dedup_by(|a, b| a.to == b.to && a.link_type == b.link_type && a.source == b.source);

    edges
}
