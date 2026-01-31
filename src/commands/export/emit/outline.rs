use super::bundle::export_bundle;
use super::links::{build_link_maps, rewrite_links};
use super::markdown_utils::add_compaction_metadata;
use super::ExportContext;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::Note;
use std::collections::{HashMap, HashSet};

pub fn export_outline(ctx: &ExportContext) -> Result<String> {
    export_outline_impl(ctx, !ctx.cli.no_resolve_compaction)
}

fn export_outline_impl(ctx: &ExportContext, resolve_compaction: bool) -> Result<String> {
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

    // Export notes in MOC link order
    let ordered_ids = extract_moc_ordered_ids(&moc, resolve_compaction, ctx.compaction_ctx);

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let compaction_note_map = CompactionContext::build_note_map(ctx.all_notes);

    // Create a lookup for fast note access
    let note_map: HashMap<_, _> = ctx.notes.iter().map(|n| (n.id(), n)).collect();

    let mut seen_ids = HashSet::new();

    for target_id in ordered_ids {
        if !seen_ids.insert(target_id.clone()) {
            continue;
        }
        if let Some(note) = note_map.get(target_id.as_str()) {
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
    }

    Ok(output)
}

fn extract_moc_ordered_ids(
    moc: &Note,
    resolve_compaction: bool,
    compaction_ctx: &CompactionContext,
) -> Vec<String> {
    let mut ordered_ids = Vec::new();
    let mut seen_ids = HashSet::new();

    // 1. Extract typed links from frontmatter first (preserves order)
    for typed_link in &moc.frontmatter.links {
        let mut target_id = typed_link.id.clone();
        if resolve_compaction {
            if let Ok(canon) = compaction_ctx.canon(&target_id) {
                target_id = canon;
            }
        }
        if seen_ids.insert(target_id.clone()) {
            ordered_ids.push(target_id);
        }
    }

    // 2. Extract wiki links from body
    let wiki_link_re = match regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]") {
        Ok(re) => re,
        Err(_) => return ordered_ids,
    };

    for cap in wiki_link_re.captures_iter(&moc.body) {
        let target = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        if target.is_empty() {
            continue;
        }
        let mut target_id = target.to_string();
        if resolve_compaction {
            if let Ok(canon) = compaction_ctx.canon(&target_id) {
                target_id = canon;
            }
        }
        if seen_ids.insert(target_id.clone()) {
            ordered_ids.push(target_id);
        }
    }

    ordered_ids
}
