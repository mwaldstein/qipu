use super::{ExportContext, ExportMode};
use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::{Note, Source};
use qipu_core::records::escape_quotes;

pub fn export_records(ctx: &ExportContext) -> Result<String> {
    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(ctx.all_notes);

    let mut output = String::new();

    // Header line
    let mode_str = match ctx.options.mode {
        ExportMode::Bundle => "export.bundle",
        ExportMode::Outline => "export.outline",
        ExportMode::Bibliography => "export.bibliography",
    };

    output.push_str(&format!(
        "H qipu=1 records=1 store={} mode={} notes={} truncated=false\n",
        ctx.store.root().display(),
        mode_str,
        ctx.notes.len()
    ));

    if ctx.options.mode == ExportMode::Bibliography {
        export_bibliography_records(&mut output, ctx.notes);
        return Ok(output);
    }

    for note in ctx.notes {
        export_note_record(&mut output, note, ctx.cli, ctx.compaction_ctx, &note_map);
    }

    Ok(output)
}

fn export_bibliography_records(output: &mut String, notes: &[Note]) {
    let mut all_sources: Vec<(&Note, Source)> = Vec::new();

    for note in notes {
        if let Some(source_url) = &note.frontmatter.source {
            all_sources.push((
                note,
                Source {
                    url: source_url.clone(),
                    title: None,
                    accessed: None,
                },
            ));
        }
        for source in &note.frontmatter.sources {
            all_sources.push((note, source.clone()));
        }
    }

    all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

    for (note, source) in all_sources {
        let title = source.title.as_deref().unwrap_or(&source.url);
        let accessed = source.accessed.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "D source url={} title=\"{}\" accessed={} from={}\n",
            source.url,
            escape_quotes(title),
            accessed,
            note.id()
        ));
    }
}

fn export_note_record(
    output: &mut String,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) {
    let tags_csv = note.frontmatter.format_tags();

    let mut annotations = String::new();
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count > 0 {
        annotations.push_str(&format!(" compacts={}", compacts_count));
        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            annotations.push_str(&format!(" compaction={:.0}%", pct));
        }
    }

    output.push_str(&format!(
        "N {} {} \"{}\" tags={}{}\n",
        note.id(),
        note.note_type(),
        escape_quotes(note.title()),
        tags_csv,
        annotations
    ));

    if cli.with_compaction_ids && compacts_count > 0 {
        export_compacted_ids(output, note, cli, compaction_ctx, compacts_count);
    }

    let summary = note.summary();
    if !summary.is_empty() {
        output.push_str(&format!("S {} {}\n", note.id(), summary));
    }

    if !note.body.is_empty() {
        output.push_str(&format!("B {}\n", note.id()));
        output.push_str(&note.body);
        if !note.body.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("B-END\n");
    }
}

fn export_compacted_ids(
    output: &mut String,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    compacts_count: usize,
) {
    let depth = cli.compaction_depth.unwrap_or(1);
    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(&note.frontmatter.id, depth, cli.compaction_max_nodes)
    {
        for id in &ids {
            output.push_str(&format!("D compacted {} from={}\n", id, note.id()));
        }
        if truncated {
            let max = cli.compaction_max_nodes.unwrap_or(ids.len());
            output.push_str(&format!(
                "D compacted_truncated max={} total={}\n",
                max, compacts_count
            ));
        }
    }
}
