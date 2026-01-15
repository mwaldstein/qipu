//! `qipu export` command - export notes to a single document
//!
//! Per spec (specs/export.md):
//! - Export modes: bundle (concatenate), outline (MOC-first), bibliography (sources only)
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Deterministic ordering: MOC order or (created_at, id)
//! - Output: stdout by default, or `--output <path>` for file

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::{search, Index, IndexBuilder};
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Export mode
#[derive(Debug, Clone, PartialEq)]
pub enum ExportMode {
    /// Bundle export: concatenate notes with metadata headers
    Bundle,
    /// Outline export: use MOC ordering
    Outline,
    /// Bibliography export: extract sources
    Bibliography,
}

impl ExportMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bundle" => Ok(ExportMode::Bundle),
            "outline" => Ok(ExportMode::Outline),
            "bibliography" | "bib" => Ok(ExportMode::Bibliography),
            _ => Err(QipuError::Other(format!(
                "invalid export mode '{}'. Valid modes: bundle, outline, bibliography",
                s
            ))),
        }
    }
}

/// Options for the export command
pub struct ExportOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub output: Option<&'a std::path::Path>,
    pub mode: ExportMode,
}

/// Execute the export command
pub fn execute(cli: &Cli, store: &Store, options: ExportOptions) -> Result<()> {
    use crate::cli::OutputFormat;

    // Build or load index for searching
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context for resolved view + annotations
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    // Collect notes based on selection criteria
    let mut selected_notes = collect_notes(store, &index, &all_notes, &options)?;

    // Apply compaction resolution unless disabled
    if !cli.no_resolve_compaction {
        selected_notes = resolve_compaction_notes(store, &compaction_ctx, selected_notes)?;
    }

    // Sort notes deterministically (by created, then by id)
    // Per spec: "For tag/query-driven exports: sort by (created_at, id)"
    sort_notes_by_created_id(&mut selected_notes);

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            eprintln!("warning: no notes selected for export");
        }
        return Ok(());
    }

    // Generate output based on format and mode
    let output_content = match cli.format {
        OutputFormat::Human => {
            // Generate markdown output based on export mode
            match options.mode {
                ExportMode::Bundle => {
                    export_bundle(&selected_notes, store, cli, &compaction_ctx, &all_notes)?
                }
                ExportMode::Outline => export_outline(
                    &selected_notes,
                    store,
                    &index,
                    options.moc_id,
                    cli,
                    &compaction_ctx,
                    !cli.no_resolve_compaction,
                    &all_notes,
                )?,
                ExportMode::Bibliography => export_bibliography(&selected_notes)?,
            }
        }
        OutputFormat::Json => {
            // JSON output: list of notes with metadata
            export_json(
                &selected_notes,
                store,
                &options,
                cli,
                &compaction_ctx,
                &all_notes,
            )?
        }
        OutputFormat::Records => {
            // Records output: low-overhead format
            export_records(
                &selected_notes,
                store,
                &options,
                cli,
                &compaction_ctx,
                &all_notes,
            )?
        }
    };

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::Other(format!("failed to create output file: {}", e)))?;
        file.write_all(output_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to output file: {}", e)))?;

        if cli.verbose && !cli.quiet {
            eprintln!(
                "exported {} notes to {}",
                selected_notes.len(),
                output_path.display()
            );
        }
    } else {
        print!("{}", output_content);
    }

    Ok(())
}

/// Collect notes based on selection criteria
fn collect_notes(
    store: &Store,
    index: &Index,
    all_notes: &[Note],
    options: &ExportOptions,
) -> Result<Vec<Note>> {
    let mut selected_notes: Vec<Note> = Vec::new();
    let mut seen_ids = HashSet::new();

    // Selection by explicit note IDs
    for id in options.note_ids {
        if seen_ids.insert(id.clone()) {
            match store.get_note(id) {
                Ok(note) => selected_notes.push(note),
                Err(_) => {
                    return Err(QipuError::NoteNotFound { id: id.clone() });
                }
            }
        }
    }

    // Selection by tag
    if let Some(tag_name) = options.tag {
        for note in all_notes {
            if note.frontmatter.tags.contains(&tag_name.to_string())
                && seen_ids.insert(note.id().to_string())
            {
                selected_notes.push(note.clone());
            }
        }
    }

    // Selection by MOC (same logic as context command)
    if let Some(moc_id) = options.moc_id {
        let linked_notes = get_moc_linked_notes(store, index, moc_id)?;
        for note in linked_notes {
            if seen_ids.insert(note.id().to_string()) {
                selected_notes.push(note);
            }
        }
    }

    // Selection by query
    if let Some(q) = options.query {
        let results = search(store, index, q, None, None)?;
        for result in results {
            if seen_ids.insert(result.id.clone()) {
                if let Ok(note) = store.get_note(&result.id) {
                    selected_notes.push(note);
                }
            }
        }
    }

    // If no selection criteria provided, return error
    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
    {
        return Err(QipuError::Other(
            "no selection criteria provided. Use --note, --tag, --moc, or --query".to_string(),
        ));
    }

    Ok(selected_notes)
}

fn resolve_compaction_notes(
    store: &Store,
    compaction_ctx: &CompactionContext,
    notes: Vec<Note>,
) -> Result<Vec<Note>> {
    let mut resolved = Vec::new();
    let mut seen_ids = HashSet::new();

    for note in notes {
        let canonical_id = compaction_ctx.canon(note.id())?;
        if seen_ids.insert(canonical_id.clone()) {
            if canonical_id == note.id() {
                resolved.push(note);
            } else {
                resolved.push(store.get_note(&canonical_id)?);
            }
        }
    }

    Ok(resolved)
}

fn sort_notes_by_created_id(notes: &mut [Note]) {
    notes.sort_by(|a, b| {
        match (&a.frontmatter.created, &b.frontmatter.created) {
            (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.id().cmp(b.id()))
    });
}

/// Get notes linked from a MOC (direct links only, not transitive)
fn get_moc_linked_notes(store: &Store, index: &Index, moc_id: &str) -> Result<Vec<Note>> {
    let moc = store.get_note(moc_id)?;

    // Get all outbound links from the MOC
    let edges = index.get_outbound_edges(moc.id());
    let mut linked_notes = Vec::new();

    for edge in edges {
        if let Ok(note) = store.get_note(&edge.to) {
            linked_notes.push(note);
        }
    }

    Ok(linked_notes)
}

/// Export mode: Bundle
/// Concatenate notes with metadata headers
fn export_bundle(
    notes: &[Note],
    _store: &Store,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# Exported Notes\n\n");

    for (i, note) in notes.iter().enumerate() {
        if i > 0 {
            output.push_str("\n---\n\n");
        }

        // Note header
        output.push_str(&format!("## Note: {} ({})\n\n", note.title(), note.id()));

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

        if let Some(path) = &note.path {
            output.push_str(&format!("**Path:** {}\n\n", path.display()));
        }

        // Compaction annotations for digest notes
        let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
        if compacts_count > 0 {
            output.push_str(&format!("**Compaction:** compacts={}", compacts_count));

            if let Some(pct) = compaction_ctx.get_compaction_pct(note, all_notes) {
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

        // Sources
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

        // Body content
        output.push_str(&note.body);
        output.push('\n');
    }

    Ok(output)
}

/// Export mode: Outline
/// Use MOC ordering for export
fn export_outline(
    notes: &[Note],
    store: &Store,
    index: &Index,
    moc_id: Option<&str>,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    resolve_compaction: bool,
    all_notes: &[Note],
) -> Result<String> {
    // If no MOC provided, fall back to bundle mode with warning
    let Some(moc_id) = moc_id else {
        if cli.verbose && !cli.quiet {
            eprintln!("warning: outline mode requires --moc flag, falling back to bundle mode");
        }
        return export_bundle(notes, store, cli, compaction_ctx, all_notes);
    };

    let moc = store.get_note(moc_id)?;
    let mut output = String::new();

    // Title from MOC
    output.push_str(&format!("# {}\n\n", moc.title()));

    // MOC body as introduction
    output.push_str(&moc.body);
    output.push_str("\n\n");

    // Export notes in MOC link order
    let edges = index.get_outbound_edges(moc.id());

    // Create a lookup for fast note access
    let note_map: std::collections::HashMap<_, _> = notes.iter().map(|n| (n.id(), n)).collect();

    // Sort edges to get deterministic order (by target id)
    let mut sorted_edges = edges;
    sorted_edges.sort_by_key(|edge| &edge.to);

    let mut ordered_ids = Vec::new();
    let mut seen_ids = HashSet::new();

    for edge in sorted_edges {
        let mut target_id = edge.to.clone();
        if resolve_compaction {
            target_id = compaction_ctx.canon(&target_id)?;
        }
        if seen_ids.insert(target_id.clone()) {
            ordered_ids.push(target_id);
        }
    }

    for target_id in ordered_ids {
        if let Some(note) = note_map.get(target_id.as_str()) {
            output.push_str("\n---\n\n");
            output.push_str(&format!("## {} ({})\n\n", note.title(), note.id()));

            // Minimal metadata for outline mode
            if !note.frontmatter.tags.is_empty() {
                output.push_str(&format!(
                    "**Tags:** {}\n\n",
                    note.frontmatter.tags.join(", ")
                ));
            }

            // Compaction annotations for digest notes
            let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
            if compacts_count > 0 {
                output.push_str(&format!("**Compaction:** compacts={}", compacts_count));
                if let Some(pct) = compaction_ctx.get_compaction_pct(note, all_notes) {
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

            output.push_str(&note.body);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Export mode: Bibliography
/// Extract sources from notes
fn export_bibliography(notes: &[Note]) -> Result<String> {
    let mut output = String::new();
    output.push_str("# Bibliography\n\n");

    let mut all_sources = Vec::new();

    // Collect all sources from all notes
    for note in notes {
        for source in &note.frontmatter.sources {
            all_sources.push((note, source));
        }
    }

    if all_sources.is_empty() {
        output.push_str("*No sources found in selected notes.*\n");
        return Ok(output);
    }

    // Sort sources by URL for deterministic output
    all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

    // Group by note or output as flat list
    // For now, implement flat list (simpler)
    for (note, source) in all_sources {
        if let Some(title) = &source.title {
            output.push_str(&format!("- [{}]({})", title, source.url));
        } else {
            output.push_str(&format!("- {}", source.url));
        }

        if let Some(accessed) = &source.accessed {
            output.push_str(&format!(" (accessed {})", accessed));
        }

        output.push_str(&format!(" — from: {}", note.title()));
        output.push('\n');
    }

    Ok(output)
}

/// Export in JSON format
fn export_json(
    notes: &[Note],
    store: &Store,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    let mode_str = match options.mode {
        ExportMode::Bundle => "bundle",
        ExportMode::Outline => "outline",
        ExportMode::Bibliography => "bibliography",
    };

    let output = serde_json::json!({
        "store": store.root().display().to_string(),
        "mode": mode_str,
        "notes": notes
            .iter()
            .map(|note| {
                let mut obj = serde_json::json!({
                    "id": note.id(),
                    "title": note.title(),
                    "type": note.note_type().to_string(),
                    "tags": note.frontmatter.tags,
                    "path": note.path.as_ref().map(|p| p.display().to_string()),
                    "created": note.frontmatter.created,
                    "updated": note.frontmatter.updated,
                    "content": note.body,
                    "sources": note.frontmatter.sources.iter().map(|s| {
                        let mut obj = serde_json::json!({
                            "url": s.url,
                        });
                        if let Some(title) = &s.title {
                            obj["title"] = serde_json::json!(title);
                        }
                        if let Some(accessed) = &s.accessed {
                            obj["accessed"] = serde_json::json!(accessed);
                        }
                        obj
                    }).collect::<Vec<_>>(),
                });

                // Add compaction annotations for digest notes
                let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                if compacts_count > 0 {
                    if let Some(obj_mut) = obj.as_object_mut() {
                        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));
                        if let Some(pct) = compaction_ctx.get_compaction_pct(note, all_notes) {
                            obj_mut.insert(
                                "compaction_pct".to_string(),
                                serde_json::json!(format!("{:.1}", pct)),
                            );
                        }

                        if cli.with_compaction_ids {
                            let depth = cli.compaction_depth.unwrap_or(1);
                            if let Some((ids, _truncated)) = compaction_ctx.get_compacted_ids(
                                &note.frontmatter.id,
                                depth,
                                cli.compaction_max_nodes,
                            ) {
                                obj_mut.insert(
                                    "compacted_ids".to_string(),
                                    serde_json::json!(ids),
                                );
                            }
                        }
                    }
                }

                obj
            })
            .collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&output)?)
}

/// Export in Records format (low-overhead for context injection)
fn export_records(
    notes: &[Note],
    store: &Store,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    let mut output = String::new();

    // Header line per spec (specs/records-output.md)
    let mode_str = match options.mode {
        ExportMode::Bundle => "export.bundle",
        ExportMode::Outline => "export.outline",
        ExportMode::Bibliography => "export.bibliography",
    };

    output.push_str(&format!(
        "H qipu=1 records=1 store={} mode={} notes={} truncated=false\n",
        store.root().display(),
        mode_str,
        notes.len()
    ));

    // For bibliography mode, output is different
    if options.mode == ExportMode::Bibliography {
        // Collect all sources
        let mut all_sources = Vec::new();
        for note in notes {
            for source in &note.frontmatter.sources {
                all_sources.push((note, source));
            }
        }

        // Sort sources by URL for deterministic output
        all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

        // Output source lines (D for data/diagnostic lines)
        for (note, source) in all_sources {
            let title = source.title.as_deref().unwrap_or(&source.url);
            let accessed = source.accessed.as_deref().unwrap_or("-");
            output.push_str(&format!(
                "D source url={} title=\"{}\" accessed={} from={}\n",
                source.url,
                title,
                accessed,
                note.id()
            ));
        }

        return Ok(output);
    }

    // For bundle/outline modes: output notes with metadata and summaries
    for note in notes {
        let tags_csv = if note.frontmatter.tags.is_empty() {
            "-".to_string()
        } else {
            note.frontmatter.tags.join(",")
        };

        // Note metadata line with compaction annotations
        let mut annotations = String::new();
        let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
        if compacts_count > 0 {
            annotations.push_str(&format!(" compacts={}", compacts_count));
            if let Some(pct) = compaction_ctx.get_compaction_pct(note, all_notes) {
                annotations.push_str(&format!(" compaction={:.0}%", pct));
            }
        }

        output.push_str(&format!(
            "N {} {} \"{}\" tags={}{}\n",
            note.id(),
            note.note_type(),
            note.title(),
            tags_csv,
            annotations
        ));

        // Show compacted IDs if --with-compaction-ids is set
        if cli.with_compaction_ids && compacts_count > 0 {
            let depth = cli.compaction_depth.unwrap_or(1);
            if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                &note.frontmatter.id,
                depth,
                cli.compaction_max_nodes,
            ) {
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

        // Summary line (if available)
        let summary = note.summary();
        if !summary.is_empty() {
            output.push_str(&format!("S {} {}\n", note.id(), summary));
        }

        // Body content (optional, could be controlled by --with-body flag in future)
        // For now, include bodies in export since that's the primary use case
        if !note.body.is_empty() {
            output.push_str(&format!("B {}\n", note.id()));
            output.push_str(&note.body);
            if !note.body.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("B-END\n");
        }
    }

    Ok(output)
}
