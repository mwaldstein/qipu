use crate::lib::error::Result;
use crate::lib::note::Note;

pub fn export_bibliography(notes: &[Note]) -> Result<String> {
    let mut output = String::new();
    output.push_str("# Bibliography\n\n");

    let mut all_sources = Vec::new();

    // Collect all sources from all notes
    for note in notes {
        for source in &note.frontmatter.sources {
            all_sources.push((note, source));
        }
    }

    if all_sources.is_empty() {
        output.push_str("*No sources found in selected notes.*\n");
        return Ok(output);
    }

    // Sort sources by URL for deterministic output
    all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

    for (note, source) in all_sources {
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
