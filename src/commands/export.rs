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
    // Build or load index for searching
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Collect notes based on selection criteria
    let selected_notes = collect_notes(store, &index, &options)?;

    if selected_notes.is_empty() {
        if !cli.quiet {
            eprintln!("warning: no notes selected for export");
        }
        return Ok(());
    }

    // Generate output based on export mode
    let output_content = match options.mode {
        ExportMode::Bundle => export_bundle(&selected_notes, store)?,
        ExportMode::Outline => export_outline(&selected_notes, store, &index, options.moc_id)?,
        ExportMode::Bibliography => export_bibliography(&selected_notes)?,
    };

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::Other(format!("failed to create output file: {}", e)))?;
        file.write_all(output_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to output file: {}", e)))?;

        if !cli.quiet {
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
fn collect_notes(store: &Store, index: &Index, options: &ExportOptions) -> Result<Vec<Note>> {
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
        let notes = store.list_notes()?;
        for note in notes {
            if note.frontmatter.tags.contains(&tag_name.to_string())
                && seen_ids.insert(note.id().to_string())
            {
                selected_notes.push(note);
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

    // Sort notes deterministically (by created, then by id)
    // Per spec: "For tag/query-driven exports: sort by (created_at, id)"
    selected_notes.sort_by(
        |a, b| match (&a.frontmatter.created, &b.frontmatter.created) {
            (Some(a_created), Some(b_created)) => {
                a_created.cmp(b_created).then_with(|| a.id().cmp(b.id()))
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        },
    );

    Ok(selected_notes)
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
fn export_bundle(notes: &[Note], _store: &Store) -> Result<String> {
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
) -> Result<String> {
    // If no MOC provided, fall back to bundle mode with warning
    let Some(moc_id) = moc_id else {
        eprintln!("warning: outline mode requires --moc flag, falling back to bundle mode");
        return export_bundle(notes, store);
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

    for edge in sorted_edges {
        if let Some(note) = note_map.get(edge.to.as_str()) {
            output.push_str("\n---\n\n");
            output.push_str(&format!("## {} ({})\n\n", note.title(), note.id()));

            // Minimal metadata for outline mode
            if !note.frontmatter.tags.is_empty() {
                output.push_str(&format!(
                    "**Tags:** {}\n\n",
                    note.frontmatter.tags.join(", ")
                ));
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
