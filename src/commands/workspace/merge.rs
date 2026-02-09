use crate::Cli;
use qipu_core::bail_usage;
use qipu_core::error::Result;
use qipu_core::store::Store;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::debug;

pub fn execute(
    cli: &Cli,
    root: &Path,
    source_name: &str,
    target_name: &str,
    dry_run: bool,
    strategy: &str,
    delete_source: bool,
) -> Result<()> {
    let start = Instant::now();

    if !matches!(strategy, "overwrite" | "merge-links" | "skip" | "rename") {
        bail_usage!(format!(
            "unknown merge strategy: '{}' (expected: overwrite, merge-links, skip, or rename)",
            strategy
        ));
    }

    if cli.verbose {
        debug!(source_name, target_name, strategy, dry_run, "merge_params");
    }

    let primary_store = Store::discover(root)?;

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discovered_stores");
    }

    let source_store = if source_name == "." {
        Store::discover(root)?
    } else {
        let path = primary_store.workspaces_dir().join(source_name);
        Store::open(&path)?
    };

    let target_store = if target_name == "." {
        Store::discover(root)?
    } else {
        let path = primary_store.workspaces_dir().join(target_name);
        Store::open(&path)?
    };

    let source_notes = source_store.list_notes()?;
    let target_notes_ids = target_store.existing_ids()?;

    let mut conflicts: Vec<(String, &str)> = Vec::new();
    let mut additions: Vec<String> = Vec::new();
    let mut id_mappings: HashMap<String, String> = HashMap::new();

    // For rename strategy, first pass: build ID mappings for conflicts
    if strategy == "rename" {
        for note in &source_notes {
            let id: String = note.id().to_string();
            if target_notes_ids.contains(&id) {
                // Generate a new unique ID by appending a numeric suffix
                let mut suffix = 1;
                let new_id = loop {
                    let candidate = format!("{}-{}", id, suffix);
                    if !target_notes_ids.contains(&candidate)
                        && !id_mappings.values().any(|v| v == &candidate)
                    {
                        break candidate;
                    }
                    suffix += 1;
                };
                id_mappings.insert(id.clone(), new_id.clone());
                conflicts.push((id.clone(), "rename"));
            }
        }
    }

    // Second pass: process notes
    // For rename strategy, process conflicts first to ensure renamed IDs exist
    // before other notes reference them
    if strategy == "rename" && !dry_run {
        // Process renamed notes (conflicts) first
        for note in &source_notes {
            let id: String = note.id().to_string();
            if target_notes_ids.contains(&id) {
                let new_id = id_mappings.get(&id).unwrap();
                additions.push(new_id.clone());
                copy_note_with_rename(note, &source_store, &target_store, new_id, &id_mappings)?;
            }
        }
        // Process non-conflicting notes second
        for note in &source_notes {
            let id: String = note.id().to_string();
            if !target_notes_ids.contains(&id) {
                additions.push(id.clone());
                copy_note(note, &source_store, &target_store, &id_mappings)?;
            }
        }
    } else {
        // Non-rename strategies or dry-run: process in original order
        for note in &source_notes {
            let id: String = note.id().to_string();
            if target_notes_ids.contains(&id) {
                let action = strategy;
                if strategy != "rename" {
                    conflicts.push((id.clone(), action));
                }
                if !dry_run {
                    match strategy {
                        "overwrite" => {
                            let target_note = target_store.get_note(&id)?;
                            if let Some(path) = target_note.path {
                                let _ = std::fs::remove_file(path);
                            }
                            copy_note(note, &source_store, &target_store, &id_mappings)?;
                        }
                        "merge-links" => {
                            let mut target_note = target_store.get_note(&id)?;
                            for link in &note.frontmatter.links {
                                if !target_note.frontmatter.links.contains(link) {
                                    target_note.frontmatter.links.push(link.clone());
                                }
                            }
                            target_store.save_note(&mut target_note)?;
                        }
                        "skip" => {
                            // Intentionally do nothing - note is skipped
                        }
                        _ => {}
                    }
                }
            } else {
                additions.push(id.clone());
                if !dry_run {
                    copy_note(note, &source_store, &target_store, &id_mappings)?;
                }
            }
        }
    }

    if cli.verbose {
        debug!(
            conflicts = conflicts.len(),
            additions = additions.len(),
            "merge_notes_processed"
        );
    }

    if dry_run {
        println!(
            "Dry-run: Workspace merge from '{}' to '{}'",
            source_name, target_name
        );
        println!();
        println!("Notes to add: {}", additions.len());
        if !additions.is_empty() {
            for id in &additions {
                println!("  + {}", id);
            }
        }
        println!();
        println!("Conflicts: {}", conflicts.len());
        if !conflicts.is_empty() {
            println!("Strategy: {}", strategy);
            for (id, action) in &conflicts {
                println!("  {} [{}]", id, action);
            }
        }
        return Ok(());
    }

    // Post-merge integrity validation (per specs/workspaces.md)
    if cli.verbose {
        debug!("running_post_merge_validation");
    }

    let validation_result =
        crate::commands::doctor::execute(cli, &target_store, false, false, 0.8, false)?;

    if cli.verbose {
        debug!(
            errors = validation_result.error_count,
            warnings = validation_result.warning_count,
            "post_merge_validation_complete"
        );
    }

    // Report validation issues if any
    if (validation_result.error_count > 0 || validation_result.warning_count > 0) && !cli.quiet {
        println!();
        println!("Post-merge validation found issues:");
        println!(
            "  Errors: {}, Warnings: {}",
            validation_result.error_count, validation_result.warning_count
        );
    }

    if delete_source && source_name != "." {
        std::fs::remove_dir_all(source_store.root())?;
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "merge_complete");
    }

    if !cli.quiet {
        println!("Merged workspace '{}' into '{}'", source_name, target_name);
        println!("  Added: {} notes", additions.len());
        if !conflicts.is_empty() {
            println!("  Conflicts resolved: {} notes", conflicts.len());
        }
    }

    Ok(())
}

fn copy_note(
    note: &qipu_core::note::Note,
    src: &Store,
    dst: &Store,
    id_mappings: &HashMap<String, String>,
) -> Result<()> {
    let mut new_note = note.clone();

    // Rewrite links based on ID mappings (for rename strategy)
    for link in &mut new_note.frontmatter.links {
        if let Some(new_target_id) = id_mappings.get(&link.id) {
            link.id = new_target_id.clone();
        }
    }

    // Rewrite file references in body based on ID mappings
    new_note.body = rewrite_body_file_references(&new_note.body, id_mappings);

    // Determine target directory
    let target_dir = if new_note.note_type().is_moc() {
        dst.mocs_dir()
    } else {
        dst.notes_dir()
    };

    // Determine file path
    let id_obj = qipu_core::id::NoteId::new_unchecked(new_note.id().to_string());
    let file_name = qipu_core::id::filename(&id_obj, new_note.title());
    let file_path = target_dir.join(&file_name);

    new_note.path = Some(file_path);

    dst.save_note(&mut new_note)?;

    // Copy any referenced attachments from source to destination
    crate::commands::helpers::copy_note_attachments(&note.body, src, dst)?;

    Ok(())
}

fn copy_note_with_rename(
    note: &qipu_core::note::Note,
    src: &Store,
    dst: &Store,
    new_id: &str,
    id_mappings: &HashMap<String, String>,
) -> Result<()> {
    let mut new_note = note.clone();

    // Update the note's ID
    new_note.frontmatter.id = new_id.to_string();

    // Rewrite links based on ID mappings (for rename strategy)
    for link in &mut new_note.frontmatter.links {
        if let Some(new_target_id) = id_mappings.get(&link.id) {
            link.id = new_target_id.clone();
        }
    }

    // Rewrite file references in body based on ID mappings
    new_note.body = rewrite_body_file_references(&new_note.body, id_mappings);

    // Determine target directory
    let target_dir = if new_note.note_type().is_moc() {
        dst.mocs_dir()
    } else {
        dst.notes_dir()
    };

    // Determine file path
    let id_obj = qipu_core::id::NoteId::new_unchecked(new_id.to_string());
    let file_name = qipu_core::id::filename(&id_obj, new_note.title());
    let file_path = target_dir.join(&file_name);

    new_note.path = Some(file_path);

    dst.save_note(&mut new_note)?;

    // Copy any referenced attachments from source to destination
    crate::commands::helpers::copy_note_attachments(&note.body, src, dst)?;

    Ok(())
}

/// Rewrite markdown file references in note body when IDs are renamed.
/// Handles patterns like:
/// - Direct ID: [text](qp-xxxx)
/// - Path with ID: [text](./qp-xxxx-slug.md) or [text](../other/qp-xxxx-slug.md)
fn rewrite_body_file_references(body: &str, id_mappings: &HashMap<String, String>) -> String {
    use regex::Regex;

    let md_link_re = match Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(_) => return body.to_string(),
    };

    md_link_re
        .replace_all(body, |caps: &regex::Captures| {
            let label = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let target = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();

            // Skip external URLs and anchors
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with('#')
            {
                return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
            }

            // Try to find and rewrite ID in the target
            let new_target = rewrite_target_id(target, id_mappings);

            if new_target != target {
                format!("[{}]({})", label, new_target)
            } else {
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }
        })
        .to_string()
}

/// Rewrite an ID within a target path if it matches a mapping.
/// Handles: qp-xxxx, ./qp-xxxx-slug.md, ../other/qp-xxxx-slug.md
fn rewrite_target_id(target: &str, id_mappings: &HashMap<String, String>) -> String {
    // Try to extract ID from the target
    let id = if target.starts_with("qp-") {
        // Direct ID: qp-xxxx
        target.split('-').take(2).collect::<Vec<_>>().join("-")
    } else if let Some(start) = target.find("qp-") {
        // Path containing ID: ./qp-xxxx-slug.md or ../other/qp-xxxx-slug.md
        let rest = &target[start..];
        // Extract the ID portion (qp-xxxx)
        let end = rest
            .find('-')
            .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
        match end {
            Some(end) => rest[..end].to_string(),
            None => rest.trim_end_matches(".md").to_string(),
        }
    } else {
        return target.to_string();
    };

    // If this ID is being renamed, rewrite the target
    if let Some(new_id) = id_mappings.get(&id) {
        // Replace the old ID with the new one
        target.replace(&id, new_id)
    } else {
        target.to_string()
    }
}
