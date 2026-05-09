use super::bundle::export_bundle;
use super::links::{build_link_maps, rewrite_links};
use super::markdown_utils::add_compaction_metadata;
use super::ExportContext;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;

pub fn export_outline(ctx: &ExportContext) -> Result<String> {
    export_outline_impl(ctx)
}

fn export_outline_impl(ctx: &ExportContext) -> Result<String> {
    // If no MOC provided, fall back to bundle mode with warning
    let Some(moc_id) = ctx.options.moc_id else {
        if ctx.cli.verbose && !ctx.cli.quiet {
            tracing::info!("outline mode requires --moc flag, falling back to bundle mode");
        }
        return export_bundle(ctx);
    };

    let moc = ctx.store.get_note(moc_id)?;
    let mut output = String::new();

    // Title from MOC
    output.push_str(&format!("# {}\n\n", moc.title()));

    // MOC body as introduction
    output.push_str(&moc.body);
    output.push_str("\n\n");

    let (body_map, anchor_map) = build_link_maps(ctx.notes);

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let compaction_note_map = CompactionContext::build_note_map(ctx.all_notes);

    // ctx.notes is already selected and ordered by the shared linked-root selector.
    // The linked collection root itself is rendered above as the outline title
    // and introduction, so do not duplicate it as a child section.
    for note in ctx.notes {
        if note.id() == moc.id() {
            continue;
        }

        output.push_str("\n---\n\n");
        // Add anchor if using anchor mode
        if ctx.options.link_mode == super::super::LinkMode::Anchors {
            output.push_str(&format!(
                "<a id=\"note-{}\"></a>\n## {} ({})\n\n",
                note.id(),
                note.title(),
                note.id()
            ));
        } else {
            output.push_str(&format!("## {} ({})\n\n", note.title(), note.id()));
        }

        // Minimal metadata for outline mode
        if !note.frontmatter.tags.is_empty() {
            output.push_str(&format!(
                "**Tags:** {}\n\n",
                note.frontmatter.tags.join(", ")
            ));
        }

        // Compaction annotations for digest notes
        add_compaction_metadata(
            &mut output,
            note,
            ctx.cli,
            ctx.compaction_ctx,
            &compaction_note_map,
        );

        let body = rewrite_links(&note.body, ctx.options.link_mode, &body_map, &anchor_map);
        output.push_str(&body);
        output.push('\n');
    }

    Ok(output)
}
