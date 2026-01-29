use super::{Direction, LinkEntry, TreeOptions};
use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::graph::types::{TreeLink, TreeNote};
use qipu_core::graph::{PathResult, TreeResult};
use qipu_core::index::Index;
use qipu_core::note::Note;
use qipu_core::records::{escape_quotes, path_relative_to_cwd};
use qipu_core::store::Store;
use std::collections::HashMap;

/// Output in records format
#[allow(clippy::too_many_arguments)]
pub fn output_records(
    entries: &[LinkEntry],
    store: &Store,
    index: &Index,
    display_id: &str,
    direction: Direction,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    max_chars: Option<usize>,
) {
    let mut lines = Vec::new();

    append_note_metadata_lines(
        &mut lines,
        entries,
        store,
        index,
        cli,
        compaction_ctx,
        note_map,
    );

    append_edge_lines(&mut lines, entries, display_id);

    let header_base = build_header_base(store, display_id, direction);
    output_with_truncation(&header_base, &lines, max_chars);
}

fn collect_unique_note_ids(entries: &[LinkEntry]) -> Vec<String> {
    let mut unique_ids: Vec<String> = entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    unique_ids.sort();
    unique_ids
}

#[allow(clippy::too_many_arguments)]
fn append_note_metadata_lines(
    lines: &mut Vec<String>,
    entries: &[LinkEntry],
    store: &Store,
    index: &Index,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
) {
    let unique_ids = collect_unique_note_ids(entries);

    for link_id in &unique_ids {
        if let Some(meta) = index.get_metadata(link_id) {
            let tags_csv = if meta.tags.is_empty() {
                "-".to_string()
            } else {
                meta.tags.join(",")
            };

            let mut annotations = String::new();
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(link_id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    if let Some(map) = note_map {
                        if let Some(note) = map.get(link_id.as_str()) {
                            if let Some(pct) = ctx.get_compaction_pct(note, map) {
                                annotations.push_str(&format!(" compaction={:.0}%", pct));
                            }
                        }
                    }
                }
            }

            lines.push(format!(
                "N {} {} \"{}\" tags={} path={}{}",
                link_id,
                meta.note_type,
                escape_quotes(&meta.title),
                tags_csv,
                meta.path,
                annotations
            ));

            if let Ok(note) = store.get_note(link_id) {
                append_summary_line(lines, link_id, &note);
            }

            if cli.with_compaction_ids {
                append_compaction_lines(lines, link_id, cli, compaction_ctx);
            }
        }
    }
}

fn append_summary_line(lines: &mut Vec<String>, link_id: &str, note: &qipu_core::note::Note) {
    let summary = note.summary();
    if !summary.is_empty() {
        let summary_text = summary.lines().next().unwrap_or("").trim();
        if !summary_text.is_empty() {
            lines.push(format!("S {} {}", link_id, summary_text));
        }
    }
}

fn append_compaction_lines(
    lines: &mut Vec<String>,
    link_id: &str,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
) {
    if let Some(ctx) = compaction_ctx {
        let compacts_count = ctx.get_compacts_count(link_id);
        if compacts_count > 0 {
            let depth = cli.compaction_depth.unwrap_or(1);
            if let Some((ids, truncated)) =
                ctx.get_compacted_ids(link_id, depth, cli.compaction_max_nodes)
            {
                for id in &ids {
                    lines.push(format!("D compacted {} from={}", id, link_id));
                }
                if truncated {
                    lines.push(format!(
                        "D compacted_truncated max={} total={}",
                        cli.compaction_max_nodes.unwrap_or(ids.len()),
                        compacts_count
                    ));
                }
            }
        }
    }
}

fn append_edge_lines(lines: &mut Vec<String>, entries: &[LinkEntry], display_id: &str) {
    for entry in entries {
        let (from, to) = match entry.direction.as_str() {
            "out" => (display_id.to_string(), entry.id.clone()),
            "in" => (entry.id.clone(), display_id.to_string()),
            _ => (display_id.to_string(), entry.id.clone()),
        };
        let via_annotation = if let Some(ref via) = entry.via {
            format!(" via={}", via)
        } else {
            String::new()
        };
        lines.push(format!(
            "E {} {} {} {}{}",
            from, entry.link_type, to, entry.source, via_annotation
        ));
    }
}

fn build_header_base(store: &Store, display_id: &str, direction: Direction) -> String {
    let store_path = path_relative_to_cwd(store.root());
    format!(
        "H qipu=1 records=1 store={} mode=link.list id={} direction={} truncated=",
        store_path,
        display_id,
        match direction {
            Direction::Out => "out",
            Direction::In => "in",
            Direction::Both => "both",
        }
    )
}

fn select_lines(header_len: usize, budget: Option<usize>, lines: &[String]) -> (bool, usize) {
    if let Some(max) = budget {
        if header_len > max {
            return (true, 0);
        }
    }

    let mut used = header_len;
    let mut count = 0;
    for line in lines {
        let line_len = line.len() + 1;
        if budget.is_none_or(|max| used + line_len <= max) {
            used += line_len;
            count += 1;
        } else {
            return (true, count);
        }
    }

    (false, count)
}

fn output_with_truncation(header_base: &str, lines: &[String], max_chars: Option<usize>) {
    let header_len_false = header_base.len() + "false".len() + 1;
    let header_len_true = header_base.len() + "true".len() + 1;

    let (budget_truncated, line_count, truncated) = {
        let (budget_flag, count) = select_lines(header_len_false, max_chars, lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, max_chars, lines);
            (budget_flag, count, true)
        }
    };

    let truncated_str = if truncated || budget_truncated {
        "true"
    } else {
        "false"
    };
    println!("{}{}", header_base, truncated_str);

    for line in lines.iter().take(line_count) {
        println!("{}", line);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn output_path_records(
    result: &PathResult,
    store: &Store,
    opts: &TreeOptions,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    all_notes: &[Note],
) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    if result.found {
        append_tree_notes(
            &mut lines,
            &result.notes,
            store,
            cli,
            compaction_ctx,
            note_map,
            all_notes,
        );
        append_tree_links(&mut lines, &result.links);
    }

    let found_str = if result.found { "true" } else { "false" };
    let store_path = path_relative_to_cwd(store.root());
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
pub fn output_tree_records(
    result: &TreeResult,
    store: &Store,
    opts: &TreeOptions,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    all_notes: &[Note],
) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    append_tree_notes(
        &mut lines,
        &result.notes,
        store,
        cli,
        compaction_ctx,
        note_map,
        all_notes,
    );
    append_tree_links(&mut lines, &result.links);

    let store_path = path_relative_to_cwd(store.root());
    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.tree root={} direction={} max_hops={} truncated=",
        store_path, result.root, result.direction, result.max_hops
    );

    output_result(&header_base, &lines, budget, result.truncated);
}

#[allow(clippy::too_many_arguments)]
fn append_tree_notes(
    lines: &mut Vec<String>,
    notes: &[TreeNote],
    store: &Store,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
    all_notes: &[Note],
) {
    for note in notes {
        let tags_csv = if note.tags.is_empty() {
            "-".to_string()
        } else {
            note.tags.join(",")
        };

        let mut annotations = String::new();
        if let Some(ctx) = compaction_ctx {
            let compacts_count = ctx.get_compacts_count(&note.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                if let Some(map) = note_map {
                    if let Some(full_note) = map.get(note.id.as_str()) {
                        if let Some(pct) = ctx.get_compaction_pct(full_note, map) {
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
            escape_quotes(&note.title),
            tags_csv,
            note.path,
            annotations
        ));

        if cli.with_compaction_ids {
            append_compaction_lines(lines, &note.id, cli, compaction_ctx);
        }

        if cli.expand_compaction {
            append_compacted_notes(lines, &note.id, cli, compaction_ctx, all_notes);
        }

        if let Ok(full_note) = store.get_note(&note.id) {
            append_summary_line(lines, &note.id, &full_note);
        }
    }
}

fn append_compacted_notes(
    lines: &mut Vec<String>,
    note_id: &str,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    all_notes: &[Note],
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
                    let compacted_tags_csv = if compacted_note.frontmatter.tags.is_empty() {
                        "-".to_string()
                    } else {
                        compacted_note.frontmatter.tags.join(",")
                    };

                    let compacted_path_str = compacted_note
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "-".to_string());

                    lines.push(format!(
                        "N {} {} \"{}\" tags={} path={} compacted_from={}",
                        compacted_note.id(),
                        compacted_note.note_type(),
                        escape_quotes(compacted_note.title()),
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
                            escape_quotes(title),
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
        let (budget_flag, count) = select_lines(header_len_true, budget, lines);
        (budget_flag, count, true)
    } else {
        let (budget_flag, count) = select_lines(header_len_false, budget, lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, budget, lines);
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
