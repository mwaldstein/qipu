use super::LinkMode;
use crate::lib::note::Note;
use std::collections::HashMap;

pub fn build_link_maps(notes: &[Note]) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut body_map = HashMap::new();
    let mut anchor_map = HashMap::new();

    for note in notes {
        let id = note.id().to_string();
        let path = note
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("{}.md", id));
        let anchor = format!("#note-{}", id);
        body_map.insert(id.clone(), path);
        anchor_map.insert(id, anchor);
    }

    (body_map, anchor_map)
}

pub fn rewrite_links(
    body: &str,
    mode: LinkMode,
    body_map: &HashMap<String, String>,
    anchor_map: &HashMap<String, String>,
) -> String {
    match mode {
        LinkMode::Preserve => body.to_string(),
        LinkMode::Markdown => rewrite_wiki_links(body, body_map),
        LinkMode::Anchors => rewrite_note_links_to_anchors(body, body_map, anchor_map),
    }
}

fn rewrite_wiki_links(body: &str, body_map: &HashMap<String, String>) -> String {
    let wiki_link_re = match regex::Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]") {
        Ok(re) => re,
        Err(_) => return body.to_string(),
    };

    wiki_link_re
        .replace_all(body, |caps: &regex::Captures| {
            let target = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            if target.is_empty() {
                return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
            }
            let label = caps.get(2).map(|m| m.as_str()).unwrap_or(target).trim();
            let path = body_map.get(target).map(|p| p.as_str()).unwrap_or(target);
            format!("[{}]({})", label, path)
        })
        .to_string()
}

fn rewrite_note_links_to_anchors(
    body: &str,
    body_map: &HashMap<String, String>,
    anchor_map: &HashMap<String, String>,
) -> String {
    let wiki_link_re = match regex::Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]") {
        Ok(re) => re,
        Err(_) => return body.to_string(),
    };
    let md_link_re = match regex::Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(_) => return body.to_string(),
    };

    let rewritten = wiki_link_re.replace_all(body, |caps: &regex::Captures| {
        let target = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        if target.is_empty() {
            return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
        }
        let label = caps.get(2).map(|m| m.as_str()).unwrap_or(target).trim();
        let anchor = anchor_map.get(target).map(|a| a.as_str()).unwrap_or(target);
        format!("[{}]({})", label, anchor)
    });

    let rewritten = md_link_re.replace_all(&rewritten, |caps: &regex::Captures| {
        let label = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let target = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        if target.starts_with("#") {
            return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
        }
        let id = find_note_id_in_target(target, body_map);
        if let Some(id) = id {
            let anchor = anchor_map.get(&id).map(|a| a.as_str()).unwrap_or(target);
            format!("[{}]({})", label, anchor)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    });

    rewritten.to_string()
}

fn find_note_id_in_target(target: &str, body_map: &HashMap<String, String>) -> Option<String> {
    if target.starts_with("qp-") {
        return Some(target.split('-').take(2).collect::<Vec<_>>().join("-"));
    }
    for (id, path) in body_map {
        if target.ends_with(path) {
            return Some(id.clone());
        }
    }
    if let Some(start) = target.find("qp-") {
        let rest = &target[start..];
        if let Some(end) = rest.find('-') {
            let after = &rest[end + 1..];
            if let Some(next_dash) = after.find('-') {
                return Some(rest[..end + 1 + next_dash].to_string());
            }
        }
        return Some(rest.trim_end_matches(".md").to_string());
    }
    None
}
