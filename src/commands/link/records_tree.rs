use super::{records::LinkOutputContext, TreeOptions};
use qipu_core::graph::types::{TreeLink, TreeNote};
use qipu_core::graph::{PathResult, TreeResult};

#[allow(clippy::too_many_arguments)]
pub fn output_path_records(result: &PathResult, ctx: &LinkOutputContext, opts: &TreeOptions) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    if result.found {
        append_tree_notes(&mut lines, &result.notes, ctx);
        append_tree_links(&mut lines, &result.links);
    }

    let found_str = if result.found { "true" } else { "false" };
    let store_path = qipu_core::records::path_relative_to_cwd(ctx.store.root());
    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.path from={} to={} direction={} found={} length={} truncated=",
        store_path,
        result.from,
        result.to,
        result.direction,
        found_str,
        result.path_length
    );

    output_result(&header_base, &lines, budget, result.found);
}

#[allow(clippy::too_many_arguments)]
pub fn output_tree_records(result: &TreeResult, ctx: &LinkOutputContext, opts: &TreeOptions) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    append_tree_notes(&mut lines, &result.notes, ctx);
    append_tree_links(&mut lines, &result.links);

    let store_path = qipu_core::records::path_relative_to_cwd(ctx.store.root());
    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.tree root={} direction={} max_hops={} truncated=",
        store_path, result.root, result.direction, result.max_hops
    );

    output_result(&header_base, &lines, budget, result.truncated);
}

#[allow(clippy::too_many_arguments)]
fn append_tree_notes(lines: &mut Vec<String>, notes: &[TreeNote], ctx: &LinkOutputContext) {
    for note in notes {
        let tags_csv = if note.tags.is_empty() {
            "-".to_string()
        } else {
            note.tags.join(",")
        };

        let mut annotations = String::new();
        if let Some(compact_ctx) = ctx.compaction_ctx {
            let compacts_count = compact_ctx.get_compacts_count(&note.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                if let Some(map) = ctx.note_map {
                    if let Some(full_note) = map.get(note.id.as_str()) {
                        if let Some(pct) = compact_ctx.get_compaction_pct(full_note, map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }
                }
            }
        }

        lines.push(format!(
            "N {} {} \"{}\" tags={} path={}{}",
            note.id,
            note.note_type,
            qipu_core::records::escape_quotes(&note.title),
            tags_csv,
            note.path,
            annotations
        ));

        if ctx.cli.with_compaction_ids {
            super::records::append_compaction_lines(lines, &note.id, ctx.cli, ctx.compaction_ctx);
        }

        if ctx.cli.expand_compaction {
            append_compacted_notes(lines, &note.id, ctx.cli, ctx.compaction_ctx, ctx.all_notes);
        }

        if let Ok(full_note) = ctx.store.get_note(&note.id) {
            super::records::append_summary_line(lines, &note.id, &full_note);
        }
    }
}

fn append_compacted_notes(
    lines: &mut Vec<String>,
    note_id: &str,
    cli: &crate::cli::Cli,
    compaction_ctx: Option<&qipu_core::compaction::CompactionContext>,
    all_notes: &[qipu_core::note::Note],
) {
    if let Some(ctx) = compaction_ctx {
        let compacts_count = ctx.get_compacts_count(note_id);
        if compacts_count > 0 {
            let depth = cli.compaction_depth.unwrap_or(1);
            if let Some((compacted_notes, _truncated)) = ctx.get_compacted_notes_expanded(
                note_id,
                depth,
                cli.compaction_max_nodes,
                all_notes,
            ) {
                for compacted_note in compacted_notes {
                    let compacted_tags_csv = compacted_note.frontmatter.format_tags();

                    let compacted_path_str = compacted_note
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "-".to_string());

                    lines.push(format!(
                        "N {} {} \"{}\" tags={} path={} compacted_from={}",
                        compacted_note.id(),
                        compacted_note.note_type(),
                        qipu_core::records::escape_quotes(compacted_note.title()),
                        compacted_tags_csv,
                        compacted_path_str,
                        note_id
                    ));

                    let compacted_summary = compacted_note.summary();
                    if !compacted_summary.is_empty() {
                        let compacted_summary_line =
                            compacted_summary.lines().next().unwrap_or("").trim();
                        if !compacted_summary_line.is_empty() {
                            lines.push(format!(
                                "S {} {}",
                                compacted_note.id(),
                                compacted_summary_line
                            ));
                        }
                    }

                    for source in &compacted_note.frontmatter.sources {
                        let title = source.title.as_deref().unwrap_or(&source.url);
                        let accessed = source.accessed.as_deref().unwrap_or("-");
                        lines.push(format!(
                            "D source url={} title=\"{}\" accessed={} from={}",
                            source.url,
                            qipu_core::records::escape_quotes(title),
                            accessed,
                            compacted_note.id()
                        ));
                    }

                    lines.push(format!("B {}", compacted_note.id()));
                    lines.push(compacted_note.body.trim().to_string());
                    lines.push(format!("B-END {}", compacted_note.id()));
                }
            }
        }
    }
}

fn append_tree_links(lines: &mut Vec<String>, links: &[TreeLink]) {
    for link in links {
        let via_annotation = if let Some(ref via) = link.via {
            format!(" via={}", via)
        } else {
            String::new()
        };
        lines.push(format!(
            "E {} {} {} {}{}",
            link.from, link.link_type, link.to, link.source, via_annotation
        ));
    }
}

fn output_result(
    header_base: &str,
    lines: &[String],
    budget: Option<usize>,
    result_truncated: bool,
) {
    let header_len_false = header_base.len() + "false".len() + 1;
    let header_len_true = header_base.len() + "true".len() + 1;

    let (budget_truncated, line_count, truncated) = if result_truncated {
        let (budget_flag, count) = super::records::select_lines(header_len_true, budget, lines);
        (budget_flag, count, true)
    } else {
        let (budget_flag, count) = super::records::select_lines(header_len_false, budget, lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = super::records::select_lines(header_len_true, budget, lines);
            (budget_flag, count, true)
        }
    };

    let truncated_value = if truncated || budget_truncated {
        "true"
    } else {
        "false"
    };
    println!("{}{}", header_base, truncated_value);

    for line in lines.iter().take(line_count) {
        println!("{}", line);
    }
}
