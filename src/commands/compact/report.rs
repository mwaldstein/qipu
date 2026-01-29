use std::collections::HashSet;
use std::time::Instant;

use qipu_core::compaction::CompactionContext;
use qipu_core::error::QipuError;
use qipu_core::error::Result;
use qipu_core::format::OutputFormat;
use qipu_core::index::Index;
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;
use qipu_core::store::Store;
use tracing::debug;

use crate::cli::Cli;
use crate::commands::compact::utils::{discover_compact_store, estimate_size};

#[derive(Debug, Clone)]
struct ReportMetrics {
    direct_count: usize,
    compaction_pct: f64,
    internal_edges: usize,
    boundary_edges: usize,
    boundary_edge_ratio: f64,
    is_stale: bool,
    staleness_count: usize,
    stale_sources: Vec<String>,
    has_conflicts: bool,
    validation_errors: Vec<String>,
}

struct ReportContext {
    store: Store,
    ctx: CompactionContext,
    index: Index,
    all_notes: Vec<Note>,
    digest_id: String,
    digest_note: Note,
    direct_compacts: Vec<String>,
}

fn build_report_context(cli: &Cli, digest_id: &str) -> Result<ReportContext> {
    let store = discover_compact_store(cli)?;

    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;

    if cli.verbose {
        debug!(note_count = all_notes.len(), "build_compaction_context");
    }

    let index = IndexBuilder::new(&store).build()?;

    if cli.verbose {
        debug!("build_index");
    }

    let direct_compacts = ctx
        .get_compacted_notes(digest_id)
        .cloned()
        .unwrap_or_default();

    if direct_compacts.is_empty() {
        return Err(QipuError::Other(format!(
            "Note {} does not compact any notes",
            digest_id
        )));
    }

    let digest_note = store.get_note(digest_id)?;

    Ok(ReportContext {
        store,
        ctx,
        index,
        all_notes,
        digest_id: digest_id.to_string(),
        digest_note,
        direct_compacts,
    })
}

fn calculate_compaction_pct(ctx: &ReportContext) -> f64 {
    let digest_size = estimate_size(&ctx.digest_note);
    let mut expanded_size = 0;

    for source_id in &ctx.direct_compacts {
        if let Ok(note) = ctx.store.get_note(source_id) {
            expanded_size += estimate_size(&note);
        }
    }

    if expanded_size > 0 {
        100.0 * (1.0 - (digest_size as f64 / expanded_size as f64))
    } else {
        0.0
    }
}

fn calculate_edge_metrics(ctx: &ReportContext) -> (usize, usize, f64) {
    let compacted_set: HashSet<_> = ctx.direct_compacts.iter().cloned().collect();
    let mut internal_edges = 0;
    let mut boundary_edges = 0;

    for source_id in &ctx.direct_compacts {
        let outbound_edges = ctx.index.get_outbound_edges(source_id);
        for edge in outbound_edges {
            if compacted_set.contains(&edge.to) {
                internal_edges += 1;
            } else {
                boundary_edges += 1;
            }
        }
    }

    let total_edges = internal_edges + boundary_edges;
    let boundary_edge_ratio = if total_edges > 0 {
        (boundary_edges as f64) / (total_edges as f64)
    } else {
        0.0
    };

    (internal_edges, boundary_edges, boundary_edge_ratio)
}

fn calculate_staleness(ctx: &ReportContext) -> (bool, usize, Vec<String>) {
    let digest_updated = ctx.digest_note.frontmatter.updated;
    let mut stale_sources = Vec::new();

    for source_id in &ctx.direct_compacts {
        if let Ok(note) = ctx.store.get_note(source_id) {
            if let (Some(digest_time), Some(source_time)) =
                (digest_updated, note.frontmatter.updated)
            {
                if source_time > digest_time {
                    stale_sources.push(source_id.clone());
                }
            }
        }
    }

    let is_stale = !stale_sources.is_empty();
    let staleness_count = stale_sources.len();

    (is_stale, staleness_count, stale_sources)
}

fn calculate_metrics(ctx: &ReportContext) -> ReportMetrics {
    let compaction_pct = calculate_compaction_pct(ctx);
    let (internal_edges, boundary_edges, boundary_edge_ratio) = calculate_edge_metrics(ctx);
    let (is_stale, staleness_count, stale_sources) = calculate_staleness(ctx);

    let validation_errors = ctx.ctx.validate(&ctx.all_notes);
    let has_conflicts = !validation_errors.is_empty();

    ReportMetrics {
        direct_count: ctx.direct_compacts.len(),
        compaction_pct,
        internal_edges,
        boundary_edges,
        boundary_edge_ratio,
        is_stale,
        staleness_count,
        stale_sources,
        has_conflicts,
        validation_errors,
    }
}

fn output_human(ctx: &ReportContext, metrics: &ReportMetrics) {
    println!("Compaction Report: {}", ctx.digest_id);
    println!("=================={}", "=".repeat(ctx.digest_id.len()));
    println!();
    println!("Compaction Metrics:");
    println!("  Direct count: {}", metrics.direct_count);
    println!("  Compaction: {:.1}%", metrics.compaction_pct);
    println!();
    println!("Edge Analysis:");
    println!("  Internal edges: {}", metrics.internal_edges);
    println!("  Boundary edges: {}", metrics.boundary_edges);
    println!("  Boundary ratio: {:.2}", metrics.boundary_edge_ratio);
    println!();
    println!("Staleness:");
    if metrics.is_stale {
        println!(
            "  Status: STALE (digest older than {} sources)",
            metrics.staleness_count
        );
        println!("  Stale sources:");
        for source_id in &metrics.stale_sources {
            if let Ok(note) = ctx.store.get_note(source_id) {
                println!("    - {} ({})", note.frontmatter.title, source_id);
            }
        }
    } else {
        println!("  Status: CURRENT (digest up to date)");
    }
    println!();
    println!("Invariants:");
    if metrics.has_conflicts {
        println!("  Status: INVALID");
        println!("  Errors:");
        for err in &metrics.validation_errors {
            println!("    - {}", err);
        }
    } else {
        println!("  Status: VALID (no conflicts or cycles)");
    }
}

fn output_json(ctx: &ReportContext, metrics: &ReportMetrics) -> Result<()> {
    let output = serde_json::json!({
        "digest_id": ctx.digest_id,
        "compacts_direct_count": metrics.direct_count,
        "compaction_pct": format!("{:.1}", metrics.compaction_pct),
        "edges": {
            "internal": metrics.internal_edges,
            "boundary": metrics.boundary_edges,
            "boundary_ratio": format!("{:.2}", metrics.boundary_edge_ratio),
        },
        "staleness": {
            "is_stale": metrics.is_stale,
            "stale_count": metrics.staleness_count,
            "stale_sources": metrics.stale_sources,
        },
        "invariants": {
            "valid": !metrics.has_conflicts,
            "errors": metrics.validation_errors,
        },
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_records(ctx: &ReportContext, metrics: &ReportMetrics) {
    println!(
        "H qipu=1 records=1 mode=compact.report digest={} count={} compaction={:.1}% boundary_ratio={:.2} stale={} valid={}",
        ctx.digest_id,
        metrics.direct_count,
        metrics.compaction_pct,
        metrics.boundary_edge_ratio,
        metrics.is_stale,
        !metrics.has_conflicts
    );
    println!("D internal_edges {}", metrics.internal_edges);
    println!("D boundary_edges {}", metrics.boundary_edges);
    if metrics.is_stale {
        println!("D stale_count {}", metrics.staleness_count);
        for source_id in &metrics.stale_sources {
            println!("D stale_source {}", source_id);
        }
    }
    if metrics.has_conflicts {
        for err in &metrics.validation_errors {
            println!("D error {}", err);
        }
    }
}

fn output_report(
    ctx: &ReportContext,
    metrics: &ReportMetrics,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Human => {
            output_human(ctx, metrics);
        }
        OutputFormat::Json => {
            output_json(ctx, metrics)?;
        }
        OutputFormat::Records => {
            output_records(ctx, metrics);
        }
    }
    Ok(())
}

pub fn execute(cli: &Cli, digest_id: &str) -> Result<()> {
    let start = Instant::now();
    if cli.verbose {
        debug!(digest_id, "report_params");
    }

    let ctx = build_report_context(cli, digest_id)?;
    let metrics = calculate_metrics(&ctx);

    if cli.verbose {
        debug!(
            digest_id,
            compacts_count = ctx.direct_compacts.len(),
            compaction_pct = format!("{:.1}", metrics.compaction_pct),
            boundary_ratio = format!("{:.2}", metrics.boundary_edge_ratio),
            is_stale = metrics.is_stale,
            has_conflicts = metrics.has_conflicts,
            elapsed = ?start.elapsed(),
            "compaction_report"
        );
    }

    output_report(&ctx, &metrics, &cli.format)?;

    Ok(())
}
