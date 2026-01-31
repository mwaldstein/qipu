use super::{Direction, LinkEntry};
use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::index::Index;
use qipu_core::note::Note;
use qipu_core::records::{escape_quotes, path_relative_to_cwd};
use qipu_core::store::Store;
use std::collections::HashMap;

pub struct LinkOutputContext<'a> {
    pub store: &'a Store,
    pub index: &'a Index,
    pub cli: &'a Cli,
    pub compaction_ctx: Option<&'a CompactionContext>,
    pub note_map: Option<&'a HashMap<&'a str, &'a Note>>,
    pub max_chars: Option<usize>,
    pub all_notes: &'a [Note],
}

impl<'a> LinkOutputContext<'a> {
    pub fn new(
        store: &'a Store,
        index: &'a Index,
        cli: &'a Cli,
        compaction_ctx: Option<&'a CompactionContext>,
        note_map: Option<&'a HashMap<&'a str, &'a Note>>,
        max_chars: Option<usize>,
        all_notes: &'a [Note],
    ) -> Self {
        Self {
            store,
            index,
            cli,
            compaction_ctx,
            note_map,
            max_chars,
            all_notes,
        }
    }
}

/// Output in records format
pub fn output_records(
    entries: &[LinkEntry],
    ctx: &LinkOutputContext,
    display_id: &str,
    direction: Direction,
) {
    let mut lines = Vec::new();

    append_note_metadata_lines(&mut lines, entries, ctx);

    append_edge_lines(&mut lines, entries, display_id);

    let header_base = build_header_base(ctx.store, display_id, direction);
    output_with_truncation(&header_base, &lines, ctx.max_chars);
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

fn append_note_metadata_lines(
    lines: &mut Vec<String>,
    entries: &[LinkEntry],
    ctx: &LinkOutputContext,
) {
    let unique_ids = collect_unique_note_ids(entries);

    for link_id in &unique_ids {
        if let Some(meta) = ctx.index.get_metadata(link_id) {
            let tags_csv = if meta.tags.is_empty() {
                "-".to_string()
            } else {
                meta.tags.join(",")
            };

            let mut annotations = String::new();
            if let Some(compact_ctx) = ctx.compaction_ctx {
                let compacts_count = compact_ctx.get_compacts_count(link_id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    if let Some(map) = ctx.note_map {
                        if let Some(note) = map.get(link_id.as_str()) {
                            if let Some(pct) = compact_ctx.get_compaction_pct(note, map) {
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

            if let Ok(note) = ctx.store.get_note(link_id) {
                append_summary_line(lines, link_id, &note);
            }

            if ctx.cli.with_compaction_ids {
                append_compaction_lines(lines, link_id, ctx.cli, ctx.compaction_ctx);
            }
        }
    }
}

pub fn append_summary_line(lines: &mut Vec<String>, link_id: &str, note: &qipu_core::note::Note) {
    let summary = note.summary();
    if !summary.is_empty() {
        let summary_text = summary.lines().next().unwrap_or("").trim();
        if !summary_text.is_empty() {
            lines.push(format!("S {} {}", link_id, summary_text));
        }
    }
}

pub fn append_compaction_lines(
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

pub fn select_lines(header_len: usize, budget: Option<usize>, lines: &[String]) -> (bool, usize) {
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
