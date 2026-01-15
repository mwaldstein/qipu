use super::{ExportMode, ExportOptions};
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::store::Store;
use std::collections::HashSet;

/// Export mode: Bundle
pub fn export_bundle(
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
pub fn export_outline(
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
pub fn export_bibliography(notes: &[Note]) -> Result<String> {
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

pub fn export_json(
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

pub fn export_records(
    notes: &[Note],
    store: &Store,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    let mut output = String::new();

    // Header line
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

    if options.mode == ExportMode::Bibliography {
        let mut all_sources = Vec::new();
        for note in notes {
            for source in &note.frontmatter.sources {
                all_sources.push((note, source));
            }
        }
        all_sources.sort_by(|a, b| a.1.url.cmp(&b.1.url));

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

    for note in notes {
        let tags_csv = if note.frontmatter.tags.is_empty() {
            "-".to_string()
        } else {
            note.frontmatter.tags.join(",")
        };

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

    Ok(output)
}
