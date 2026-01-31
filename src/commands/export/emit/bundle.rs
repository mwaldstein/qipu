use super::links::{build_link_maps, rewrite_links};
use super::markdown_utils::add_compaction_metadata;
use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;

pub fn export_bundle(
    notes: &[Note],
    _store: &Store,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    let mut output = String::new();
    let (body_map, anchor_map) = build_link_maps(notes);

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(all_notes);

    output.push_str("# Exported Notes\n\n");

    for (i, note) in notes.iter().enumerate() {
        if i > 0 {
            output.push_str("\n---\n\n");
        }

        // Note header with anchor if using anchor mode
        if options.link_mode == super::LinkMode::Anchors {
            output.push_str(&format!(
                "<a id=\"note-{}\"></a>\n## Note: {} ({})\n\n",
                note.id(),
                note.title(),
                note.id()
            ));
        } else {
            output.push_str(&format!("## Note: {} ({})\n\n", note.title(), note.id()));
        }

        // Metadata
        output.push_str(&format!("**Type:** {}\n\n", note.note_type()));

        if !note.frontmatter.tags.is_empty() {
            output.push_str(&format!(
                "**Tags:** {}\n\n",
                note.frontmatter.tags.join(", ")
            ));
        }

        if let Some(created) = &note.frontmatter.created {
            output.push_str(&format!("**Created:** {}\n\n", created.to_rfc3339()));
        }

        // Compaction annotations for digest notes
        add_compaction_metadata(&mut output, note, cli, compaction_ctx, &note_map);

        // Sources
        add_sources(&mut output, note);

        // Body content
        let body = rewrite_links(&note.body, options.link_mode, &body_map, &anchor_map);
        output.push_str(&body);
        output.push('\n');
    }

    Ok(output)
}

fn add_sources(output: &mut String, note: &Note) {
    if note.frontmatter.source.is_some() || !note.frontmatter.sources.is_empty() {
        output.push_str("**Sources:**\n\n");

        if let Some(source_url) = &note.frontmatter.source {
            output.push_str(&format!("- {}\n", source_url));
        }

        for source in &note.frontmatter.sources {
            if let Some(title) = &source.title {
                output.push_str(&format!("- [{}]({})", title, source.url));
            } else {
                output.push_str(&format!("- {}", source.url));
            }
            if let Some(accessed) = &source.accessed {
                output.push_str(&format!(" (accessed {})", accessed));
            }
            output.push('\n');
        }
        output.push('\n');
    }
}
