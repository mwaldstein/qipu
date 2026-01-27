use super::frontmatter::NoteFrontmatter;
use crate::lib::error::{QipuError, Result};
use std::path::PathBuf;

/// Parse YAML frontmatter from markdown content
#[tracing::instrument(skip(content), fields(path = ?path))]
pub(crate) fn parse_frontmatter(
    content: &str,
    path: Option<&PathBuf>,
) -> Result<(NoteFrontmatter, String)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing frontmatter delimiter (---)".to_string(),
        });
    }

    let after_first = &content[3..];
    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing closing frontmatter delimiter (---)".to_string(),
        })?;

    let yaml_content = &after_first[..end_pos];
    let body_start = 3 + end_pos + 4; // Skip first ---, yaml, \n---
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    let frontmatter: NoteFrontmatter =
        serde_yaml::from_str(yaml_content).map_err(|e| QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: e.to_string(),
        })?;

    // Validate required fields
    if frontmatter.id.is_empty() {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing required field: id".to_string(),
        });
    }
    if frontmatter.title.is_empty() {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing required field: title".to_string(),
        });
    }

    Ok((frontmatter, body))
}

/// Extract content from a `## Summary` section
pub(crate) fn extract_summary_section(body: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut in_summary = false;
    let mut in_first_paragraph = false;
    let mut summary_lines = Vec::new();

    for line in lines {
        if line.starts_with("## Summary") {
            in_summary = true;
            continue;
        }
        if in_summary {
            // Stop at next heading
            if line.starts_with("## ") || line.starts_with("# ") {
                break;
            }

            // Skip leading empty lines
            if !in_first_paragraph && line.trim().is_empty() {
                continue;
            }

            // Start collecting the first paragraph
            in_first_paragraph = true;

            // Stop at the end of the first paragraph (empty line)
            if line.trim().is_empty() {
                break;
            }

            summary_lines.push(line);
        }
    }

    if summary_lines.is_empty() {
        return None;
    }

    // Join lines and trim
    let summary = summary_lines.join("\n").trim_end().to_string();

    if summary.is_empty() {
        None
    } else {
        Some(summary)
    }
}

/// Extract the first paragraph from markdown
pub(crate) fn extract_first_paragraph(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    // Skip any leading heading
    let mut lines = body.lines().peekable();
    while let Some(line) = lines.peek() {
        if line.starts_with('#') || line.trim().is_empty() {
            lines.next();
        } else {
            break;
        }
    }

    // Collect lines until empty line
    let mut para_lines = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        para_lines.push(line);
    }

    if para_lines.is_empty() {
        None
    } else {
        Some(para_lines.join(" "))
    }
}
