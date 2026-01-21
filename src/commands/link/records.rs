use super::{Direction, LinkEntry, TreeOptions};
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::graph::PathResult;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;
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

    // Generate note metadata lines
    append_note_metadata_lines(
        &mut lines,
        entries,
        store,
        index,
        cli,
        compaction_ctx,
        note_map,
    );

    // Generate edge lines
    append_edge_lines(&mut lines, entries, display_id);

    // Generate header and output with truncation handling
    let header_base = build_header_base(store, display_id, direction);
    output_with_truncation(&header_base, &lines, max_chars);
}

/// Collect unique note IDs from link entries
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

/// Append note metadata lines including summaries and compaction info
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
            // Add note metadata line with compaction annotations
            // Per spec (specs/compaction.md lines 113-122)
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

                    // Calculate compaction percentage if we have note data
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

            // Add summary line if available
            if let Ok(note) = store.get_note(link_id) {
                append_summary_line(lines, link_id, &note);
            }

            // Add compaction info if enabled
            if cli.with_compaction_ids {
                append_compaction_lines(lines, link_id, cli, compaction_ctx);
            }
        }
    }
}

/// Append summary line for a note if it has non-empty summary
fn append_summary_line(lines: &mut Vec<String>, link_id: &str, note: &crate::lib::note::Note) {
    let summary = note.summary();
    if !summary.is_empty() {
        let summary_text = summary.lines().next().unwrap_or("").trim();
        if !summary_text.is_empty() {
            lines.push(format!("S {} {}", link_id, summary_text));
        }
    }
}

/// Append compaction-related lines for a note
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

/// Append edge lines showing links between notes
fn append_edge_lines(lines: &mut Vec<String>, entries: &[LinkEntry], display_id: &str) {
    for entry in entries {
        let (from, to) = match entry.direction.as_str() {
            "out" => (display_id.to_string(), entry.id.clone()),
            "in" => (entry.id.clone(), display_id.to_string()),
            _ => (display_id.to_string(), entry.id.clone()),
        };
        lines.push(format!(
            "E {} {} {} {}",
            from, entry.link_type, to, entry.source
        ));
    }
}

/// Build the header base string for records output
fn build_header_base(store: &Store, display_id: &str, direction: Direction) -> String {
    format!(
        "H qipu=1 records=1 store={} mode=link.list id={} direction={} truncated=",
        store.root().display(),
        display_id,
        match direction {
            Direction::Out => "out",
            Direction::In => "in",
            Direction::Both => "both",
        }
    )
}

/// Calculate how many lines fit within budget
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

/// Output lines with truncation handling based on character budget
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

/// Output path in records format
#[allow(clippy::too_many_arguments)]
pub fn output_path_records(
    result: &PathResult,
    store: &Store,
    opts: &TreeOptions,
    cli: &Cli,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
) {
    let budget = opts.max_chars;
    let mut lines = Vec::new();

    if result.found {
        for note in &result.notes {
            let tags_csv = if note.tags.is_empty() {
                "-".to_string()
            } else {
                note.tags.join(",")
            };

            // Build compaction annotations for digest nodes
            // Per spec (specs/compaction.md lines 113-122)
            let mut annotations = String::new();
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&note.id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    // Calculate compaction percentage if we have note data
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
                if let Some(ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&note.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&note.id, depth, cli.compaction_max_nodes)
                        {
                            for id in &ids {
                                lines.push(format!("D compacted {} from={}", id, note.id));
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

            if let Ok(full_note) = store.get_note(&note.id) {
                let summary = full_note.summary();
                if !summary.is_empty() {
                    let summary_text = summary.lines().next().unwrap_or("").trim();
                    if !summary_text.is_empty() {
                        lines.push(format!("S {} {}", note.id, summary_text));
                    }
                }
            }
        }

        for link in &result.links {
            lines.push(format!(
                "E {} {} {} {}",
                link.from, link.link_type, link.to, link.source
            ));
        }
    }

    let found_str = if result.found { "true" } else { "false" };
    let header_base = format!(
        "H qipu=1 records=1 store={} mode=link.path from={} to={} direction={} found={} length={} truncated=",
        store.root().display(),
        result.from,
        result.to,
        result.direction,
        found_str,
        result.path_length
    );
    let header_len_false = header_base.len() + "false".len() + 1;
    let header_len_true = header_base.len() + "true".len() + 1;

    let (budget_truncated, line_count, truncated) = if result.found {
        let (budget_flag, count) = select_lines(header_len_false, budget, &lines);
        if !budget_flag && count == lines.len() {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, budget, &lines);
            (budget_flag, count, true)
        }
    } else {
        let (budget_flag, count) = select_lines(header_len_false, budget, &lines);
        if !budget_flag {
            (false, count, false)
        } else {
            let (budget_flag, count) = select_lines(header_len_true, budget, &lines);
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
