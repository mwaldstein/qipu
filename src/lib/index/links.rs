use super::types::{Edge, LinkSource};
use crate::lib::logging;
use crate::lib::note::Note;
use regex::Regex;
use std::collections::HashSet;

/// Extract all links from a note
pub(crate) fn extract_links(
    note: &Note,
    valid_ids: &HashSet<String>,
    unresolved: &mut HashSet<String>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let from_id = note.id().to_string();

    // Extract typed links from frontmatter
    for link in &note.frontmatter.links {
        let to_id = link.id.clone();
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
        }
        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: link.link_type.to_string(),
            source: LinkSource::Typed,
        });
    }

    // Extract wiki links from body: [[id]] or [[id|label]]
    let wiki_link_re = match Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]") {
        Ok(re) => re,
        Err(e) => {
            if logging::verbose_enabled() {
                eprintln!("Warning: Failed to compile wiki link regex: {}", e);
            }
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
        }
        // Inline links default to "related" type
        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: "related".to_string(),
            source: LinkSource::Inline,
        });
    }

    // Extract markdown links to qipu notes: [text](qp-xxxx) or [text](./qp-xxxx-slug.md)
    let md_link_re = match Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(e) => {
            if logging::verbose_enabled() {
                eprintln!("Warning: Failed to compile markdown link regex: {}", e);
            }
            return edges; // Return empty edges if regex fails
        }
    };
    for cap in md_link_re.captures_iter(&note.body) {
        let target = cap[2].trim();

        // Check if this looks like a qipu note reference
        let to_id = if target.starts_with("qp-") {
            // Direct ID reference
            target.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if target.contains("qp-") {
            // Path reference like ./qp-xxxx-slug.md
            if let Some(start) = target.find("qp-") {
                let rest = &target[start..];
                // Extract the ID portion (qp-xxxx)
                let end = rest
                    .find('-')
                    .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
                match end {
                    Some(end) => rest[..end].to_string(),
                    None => rest.trim_end_matches(".md").to_string(),
                }
            } else {
                continue;
            }
        } else {
            continue;
        };

        if to_id.is_empty() || !to_id.starts_with("qp-") {
            continue;
        }

        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
        }

        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: "related".to_string(),
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
