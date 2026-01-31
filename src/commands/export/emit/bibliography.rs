use qipu_core::error::{QipuError, Result};
use qipu_core::note::{Note, Source};
use serde_json::json;

/// Bibliography output format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BibFormat {
    /// Markdown format (default)
    Markdown,
    /// BibTeX format
    BibTeX,
    /// CSL JSON format
    CslJson,
}

impl BibFormat {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(BibFormat::Markdown),
            "bibtex" | "bib" => Ok(BibFormat::BibTeX),
            "csl" | "csl-json" | "json" => Ok(BibFormat::CslJson),
            _ => Err(QipuError::Other(format!(
                "invalid bibliography format '{}'. Valid formats: markdown, bibtex, csl-json",
                s
            ))),
        }
    }
}

pub fn export_bibliography(notes: &[Note], format: BibFormat) -> Result<String> {
    let mut all_sources = Vec::new();
    let mut temp_sources = Vec::new();

    // Collect all sources from all notes
    for note in notes {
        // Include singular source field if present
        if let Some(source_url) = &note.frontmatter.source {
            temp_sources.push(Source {
                url: source_url.clone(),
                title: None,
                accessed: None,
            });
        }
    }

    // Re-scan to build the final all_sources vector
    let mut temp_idx = 0;
    for note in notes {
        // Add singular source references
        if note.frontmatter.source.is_some() {
            all_sources.push((note, &temp_sources[temp_idx]));
            temp_idx += 1;
        }
        // Add sources array references
        for source in &note.frontmatter.sources {
            all_sources.push((note, source));
        }
    }

    if all_sources.is_empty() {
        return match format {
            BibFormat::Markdown => {
                Ok("# Bibliography\n\n*No sources found in selected notes.*\n".to_string())
            }
            BibFormat::BibTeX => Ok("% No sources found in selected notes.\n".to_string()),
            BibFormat::CslJson => Ok("[]".to_string()),
        };
    }

    // Sort sources by URL for deterministic output
    all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

    match format {
        BibFormat::Markdown => export_markdown(&all_sources),
        BibFormat::BibTeX => export_bibtex(&all_sources),
        BibFormat::CslJson => export_csl_json(&all_sources),
    }
}

fn export_markdown(sources: &[(&Note, &qipu_core::note::Source)]) -> Result<String> {
    let mut output = String::new();
    output.push_str("# Bibliography\n\n");

    for (note, source) in sources {
        if let Some(title) = &source.title {
            output.push_str(&format!("- [{}]({})", title, source.url));
        } else {
            output.push_str(&format!("- {}", source.url));
        }

        if let Some(accessed) = &source.accessed {
            output.push_str(&format!(" (accessed {})", accessed));
        }

        output.push_str(&format!(" — from: {}", note.title()));
        output.push('\n');
    }

    Ok(output)
}

fn export_bibtex(sources: &[(&Note, &qipu_core::note::Source)]) -> Result<String> {
    let mut output = String::new();

    for (i, (note, source)) in sources.iter().enumerate() {
        // Generate citation key from URL (use domain + index for uniqueness)
        let citation_key = generate_citation_key(&source.url, i);

        output.push_str(&format!("@misc{{{},\n", citation_key));

        // Add title if available
        if let Some(title) = &source.title {
            output.push_str(&format!("  title = {{{}}},\n", escape_bibtex(title)));
        }

        // Add URL
        output.push_str(&format!("  url = {{{}}},\n", source.url));

        // Add accessed date if available
        if let Some(accessed) = &source.accessed {
            output.push_str(&format!("  note = {{Accessed: {}}},\n", accessed));
        }

        // Add source note as additional note
        output.push_str(&format!(
            "  note = {{From: {}}}\n",
            escape_bibtex(note.title())
        ));

        output.push_str("}\n\n");
    }

    Ok(output)
}

fn export_csl_json(sources: &[(&Note, &qipu_core::note::Source)]) -> Result<String> {
    let mut items = Vec::new();

    for (i, (note, source)) in sources.iter().enumerate() {
        let citation_key = generate_citation_key(&source.url, i);

        let mut item = json!({
            "id": citation_key,
            "type": "webpage",
            "URL": source.url,
        });

        // Add title if available
        if let Some(title) = &source.title {
            item["title"] = json!(title);
        }

        // Add accessed date if available
        if let Some(accessed) = &source.accessed {
            // Parse date string (assuming format like "2024-01-15")
            if let Ok(parts) = parse_date_string(accessed) {
                item["accessed"] = json!({
                    "date-parts": [parts]
                });
            }
        }

        // Add note about source
        item["note"] = json!(format!("From: {}", note.title()));

        items.push(item);
    }

    Ok(serde_json::to_string_pretty(&items)?)
}

fn generate_citation_key(url: &str, index: usize) -> String {
    // Extract domain from URL
    let domain = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("source")
        .replace(['.', '-'], "_");

    format!("{}_{}", domain, index + 1)
}

fn escape_bibtex(s: &str) -> String {
    // Escape special BibTeX characters
    s.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('&', "\\&")
        .replace('#', "\\#")
        .replace('_', "\\_")
}

fn parse_date_string(date_str: &str) -> Result<Vec<i32>> {
    // Parse date string in format "YYYY-MM-DD" or "YYYY-MM" or "YYYY"
    let parts: Vec<&str> = date_str.split('-').collect();

    let mut date_parts = Vec::new();

    for part in parts {
        if let Ok(num) = part.parse::<i32>() {
            date_parts.push(num);
        } else {
            return Err(QipuError::Other(format!(
                "Invalid date format: {}",
                date_str
            )));
        }
    }

    if date_parts.is_empty() || date_parts.len() > 3 {
        return Err(QipuError::Other(format!(
            "Invalid date format: {}",
            date_str
        )));
    }

    Ok(date_parts)
}
