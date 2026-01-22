use super::links::{build_link_maps, rewrite_links};
use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::store::Store;

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

fn add_compaction_metadata(
    output: &mut String,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) {
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count > 0 {
        output.push_str(&format!("**Compaction:** compacts={}", compacts_count));

        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            output.push_str(&format!(" compaction={:.0}%", pct));
        }
        output.push_str("\n\n");

        if cli.with_compaction_ids {
            let depth = cli.compaction_depth.unwrap_or(1);
            if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                &note.frontmatter.id,
                depth,
                cli.compaction_max_nodes,
            ) {
                let ids_str = ids.join(", ");
                let suffix = if truncated {
                    let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                    format!(" (truncated, showing {} of {})", max, compacts_count)
                } else {
                    String::new()
                };
                output.push_str(&format!("**Compacted IDs:** {}{}\n\n", ids_str, suffix));
            }
        }
    }
}

fn add_sources(output: &mut String, note: &Note) {
    if !note.frontmatter.sources.is_empty() {
        output.push_str("**Sources:**\n\n");
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
