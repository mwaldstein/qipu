//! Markdown reference parsing helpers.

use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownLink {
    pub label: String,
    pub target: String,
}

pub fn markdown_links(body: &str) -> Vec<MarkdownLink> {
    let md_link_re = match Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };

    md_link_re
        .captures_iter(body)
        .map(|cap| MarkdownLink {
            label: cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
            target: cap
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}

pub fn is_external_or_anchor_target(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://") || target.starts_with('#')
}

pub fn extract_qipu_id_from_target(target: &str) -> Option<String> {
    if target.starts_with("qp-") {
        return Some(target.split('-').take(2).collect::<Vec<_>>().join("-"));
    }

    let start = target.find("qp-")?;
    let rest = &target[start..];
    let end = rest
        .find('-')
        .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));

    match end {
        Some(end) => Some(rest[..end].to_string()),
        None => Some(rest.trim_end_matches(".md").to_string()),
    }
}

pub fn strip_attachment_prefix<'a>(target: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes
        .iter()
        .find_map(|prefix| target.strip_prefix(prefix))
}

pub fn prefixed_attachment_targets(body: &str, prefixes: &[&str]) -> Vec<String> {
    let mut targets = Vec::new();

    for prefix in prefixes {
        let pattern = format!(r"{}([^)\s\n]+)", regex::escape(prefix));
        let Ok(re) = Regex::new(&pattern) else {
            continue;
        };

        for cap in re.captures_iter(body) {
            if let Some(filename) = cap.get(1) {
                targets.push(filename.as_str().to_string());
            }
        }
    }

    targets
}

pub fn rewrite_qipu_id_in_target(target: &str, id_mappings: &HashMap<String, String>) -> String {
    let Some(id) = extract_qipu_id_from_target(target) else {
        return target.to_string();
    };

    if let Some(new_id) = id_mappings.get(&id) {
        target.replace(&id, new_id)
    } else {
        target.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_markdown_links() {
        let links = markdown_links("See [A](qp-a) and ![B](../attachments/b.png)");
        assert_eq!(
            links,
            vec![
                MarkdownLink {
                    label: "A".to_string(),
                    target: "qp-a".to_string(),
                },
                MarkdownLink {
                    label: "B".to_string(),
                    target: "../attachments/b.png".to_string(),
                },
            ]
        );
    }

    #[test]
    fn extracts_qipu_id_from_current_target_forms() {
        assert_eq!(
            extract_qipu_id_from_target("qp-1234"),
            Some("qp-1234".to_string())
        );
        assert_eq!(
            extract_qipu_id_from_target("./qp-1234-slug.md"),
            Some("qp-1234".to_string())
        );
        assert_eq!(
            extract_qipu_id_from_target("../notes/qp-1234-slug.md"),
            Some("qp-1234".to_string())
        );
    }

    #[test]
    fn strips_configured_attachment_prefixes() {
        assert_eq!(
            strip_attachment_prefix("../attachments/a.png", &["../attachments/"]),
            Some("a.png")
        );
        assert_eq!(
            strip_attachment_prefix("./attachments/a.png", &["../attachments/"]),
            None
        );
    }

    #[test]
    fn extracts_prefixed_attachment_targets_from_bare_text() {
        assert_eq!(
            prefixed_attachment_targets(
                "See ../attachments/a.png and [B](../attachments/b.png)",
                &["../attachments/"],
            ),
            vec!["a.png".to_string(), "b.png".to_string()]
        );
    }
}
