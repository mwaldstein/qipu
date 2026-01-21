use super::bundle::export_bundle;
use super::links::{build_link_maps, rewrite_links};
use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::store::Store;
use std::collections::{HashMap, HashSet};

#[allow(clippy::too_many_arguments)]
pub fn export_outline(
    notes: &[Note],
    store: &Store,
    _index: &Index,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    resolve_compaction: bool,
    all_notes: &[Note],
) -> Result<String> {
    // If no MOC provided, fall back to bundle mode with warning
    let Some(moc_id) = options.moc_id else {
        if cli.verbose && !cli.quiet {
            tracing::info!("outline mode requires --moc flag, falling back to bundle mode");
        }
        return export_bundle(notes, store, options, cli, compaction_ctx, all_notes);
    };

    let moc = store.get_note(moc_id)?;
    let mut output = String::new();

    // Title from MOC
    output.push_str(&format!("# {}\n\n", moc.title()));

    // MOC body as introduction
    output.push_str(&moc.body);
    output.push_str("\n\n");

    let (body_map, anchor_map) = build_link_maps(notes);

    // Export notes in MOC link order
    let ordered_ids = extract_moc_ordered_ids(&moc.body, resolve_compaction, compaction_ctx);

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let compaction_note_map = CompactionContext::build_note_map(all_notes);

    // Create a lookup for fast note access
    let note_map: HashMap<_, _> = notes.iter().map(|n| (n.id(), n)).collect();

    let mut seen_ids = HashSet::new();

    for target_id in ordered_ids {
        if !seen_ids.insert(target_id.clone()) {
            continue;
        }
        if let Some(note) = note_map.get(target_id.as_str()) {
            output.push_str("\n---\n\n");
            // Add anchor if using anchor mode
            if options.link_mode == super::super::LinkMode::Anchors {
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
            add_compaction_metadata(&mut output, note, cli, compaction_ctx, &compaction_note_map);

            let body = rewrite_links(&note.body, options.link_mode, &body_map, &anchor_map);
            output.push_str(&body);
            output.push('\n');
        }
    }

    Ok(output)
}

fn extract_moc_ordered_ids(
    body: &str,
    resolve_compaction: bool,
    compaction_ctx: &CompactionContext,
) -> Vec<String> {
    let mut ordered_ids = Vec::new();
    let mut seen_ids = HashSet::new();
    let wiki_link_re = match regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]") {
        Ok(re) => re,
        Err(_) => return ordered_ids,
    };

    for cap in wiki_link_re.captures_iter(body) {
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

fn add_compaction_metadata(
    output: &mut String,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &Note>,
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
