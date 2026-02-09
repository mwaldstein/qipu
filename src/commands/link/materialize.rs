//! Link materialize command
//!
//! Converts inline links (wiki links and markdown links in the body)
//! into typed links in the note's frontmatter.

use crate::cli::{Cli, OutputFormat};
use crate::commands::link::resolve_note_id;
use qipu_core::error::Result;
use qipu_core::note::{LinkType, TypedLink};
use qipu_core::store::Store;
use regex::Regex;

/// Execute the link materialize command
///
/// Extracts inline links from the note body and adds them as typed links
/// to the frontmatter. Supports dry-run mode and optional body cleanup.
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    link_type: LinkType,
    dry_run: bool,
    remove_inline: bool,
) -> Result<()> {
    // Validate link type against active ontology
    store.config().validate_link_type(link_type.as_str())?;

    // Resolve note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load the note
    let mut note = store.get_note(&note_id)?;

    // Get valid IDs for validation
    let valid_ids = store.existing_ids().unwrap_or_default();

    // Extract inline links from body
    let inline_targets = extract_inline_links(&note.body, &valid_ids);

    if inline_targets.is_empty() {
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "no-op",
                            "note_id": note_id,
                            "message": "no inline links found"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!("No inline links found in {}", note_id);
                }
                OutputFormat::Records => {
                    println!(
                        "H qipu=1 records=1 store={} mode=link.materialize status=no-op",
                        store.root().display()
                    );
                    println!("M {} 0", note_id);
                }
            }
        }
        return Ok(());
    }

    // Build set of existing typed links to avoid duplicates
    let existing_targets: std::collections::HashSet<String> = note
        .frontmatter
        .links
        .iter()
        .filter(|l| l.link_type == link_type)
        .map(|l| l.id.clone())
        .collect();

    // Track results
    let mut added = Vec::new();
    let mut skipped = Vec::new();

    // Process each inline link
    for target_id in inline_targets {
        if existing_targets.contains(&target_id) {
            skipped.push(target_id);
            continue;
        }

        // Add as typed link
        note.frontmatter.links.push(TypedLink {
            link_type: link_type.clone(),
            id: target_id.clone(),
        });
        added.push(target_id);
    }

    let mut removed_from_body = false;

    // If not dry run, save changes
    if !dry_run && (!added.is_empty() || remove_inline) {
        // Optionally remove inline links from body
        if remove_inline {
            note.body = remove_wiki_links(&note.body);
            note.body = remove_md_links(&note.body);
            removed_from_body = true;
        }

        // Save the note
        store.save_note(&mut note)?;
    }

    // Output results
    output_result(
        cli,
        store,
        &note_id,
        &link_type,
        &added,
        &skipped,
        dry_run,
        removed_from_body,
    );

    Ok(())
}

/// Extract inline link targets from note body
fn extract_inline_links(body: &str, valid_ids: &std::collections::HashSet<String>) -> Vec<String> {
    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Extract wiki links: [[id]] or [[id|label]]
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    for cap in wiki_re.captures_iter(body) {
        let id = cap[1].trim().to_string();
        if !id.is_empty() && id.starts_with("qp-") && valid_ids.contains(&id) && !seen.contains(&id)
        {
            seen.insert(id.clone());
            targets.push(id);
        }
    }

    // Extract markdown links to qipu notes: [text](qp-xxxx) or [text](./qp-xxxx-slug.md)
    let md_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in md_re.captures_iter(body) {
        let target = cap[2].trim();

        // Skip external URLs and anchors
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('#')
        {
            continue;
        }

        // Extract ID from target
        let id = if target.starts_with("qp-") {
            // Direct ID: [text](qp-xxxx)
            Some(target.split('-').take(2).collect::<Vec<_>>().join("-"))
        } else if target.contains("qp-") {
            // Path containing ID: [text](./qp-xxxx-slug.md)
            target.find("qp-").map(|start| {
                let rest = &target[start..];
                let end = rest
                    .find('-')
                    .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
                match end {
                    Some(end) => rest[..end].to_string(),
                    None => rest.trim_end_matches(".md").to_string(),
                }
            })
        } else {
            None
        };

        if let Some(id) = id {
            if id.starts_with("qp-") && valid_ids.contains(&id) && !seen.contains(&id) {
                seen.insert(id.clone());
                targets.push(id);
            }
        }
    }

    targets
}

/// Remove wiki links from body, keeping the label text
fn remove_wiki_links(body: &str) -> String {
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    let mut result = String::new();
    let mut last_end = 0;

    for cap in wiki_re.captures_iter(body) {
        let full_match = cap.get(0).unwrap();
        result.push_str(&body[last_end..full_match.start()]);

        // Use label if provided, otherwise use the ID
        let replacement = if let Some(label_match) = cap.get(2) {
            label_match.as_str()
        } else {
            cap.get(1).unwrap().as_str()
        };
        result.push_str(replacement);

        last_end = full_match.end();
    }

    result.push_str(&body[last_end..]);
    result
}

/// Remove markdown links to qipu notes from body, keeping the link text
fn remove_md_links(body: &str) -> String {
    let md_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let mut result = String::new();
    let mut last_end = 0;

    for cap in md_re.captures_iter(body) {
        let target = cap[2].trim();

        // Only process qipu links
        if !target.starts_with("qp-") && !target.contains("qp-") {
            continue;
        }

        let full_match = cap.get(0).unwrap();
        result.push_str(&body[last_end..full_match.start()]);

        // Keep the link text
        result.push_str(&cap[1]);

        last_end = full_match.end();
    }

    result.push_str(&body[last_end..]);
    result
}

/// Output materialization results
#[allow(clippy::too_many_arguments)]
fn output_result(
    cli: &Cli,
    store: &Store,
    note_id: &str,
    link_type: &LinkType,
    added: &[String],
    skipped: &[String],
    dry_run: bool,
    removed_from_body: bool,
) {
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": if dry_run { "dry-run" } else if added.is_empty() { "unchanged" } else { "materialized" },
                    "note_id": note_id,
                    "link_type": link_type.to_string(),
                    "added": added,
                    "skipped": skipped,
                    "removed_from_body": removed_from_body,
                    "dry_run": dry_run
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                if dry_run {
                    println!("[DRY RUN] Would materialize links for {}", note_id);
                } else {
                    println!("Materialized links for {}", note_id);
                }

                if !added.is_empty() {
                    println!("  Added {} link(s) with type '{}'", added.len(), link_type);
                    for id in added {
                        println!("    - {}", id);
                    }
                }

                if !skipped.is_empty() {
                    println!("  Skipped {} duplicate(s)", skipped.len());
                    for id in skipped {
                        println!("    - {}", id);
                    }
                }

                if removed_from_body {
                    println!("  Removed inline links from body");
                }

                if dry_run {
                    println!("\nUse without --dry-run to apply changes");
                }
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=link.materialize status={}",
                store.root().display(),
                if dry_run {
                    "dry-run"
                } else if added.is_empty() {
                    "unchanged"
                } else {
                    "materialized"
                }
            );
            println!(
                "M {} {} {} {}",
                note_id,
                added.len(),
                skipped.len(),
                if removed_from_body { "cleaned" } else { "kept" }
            );
            for id in added {
                println!("A {} {} {}", note_id, link_type, id);
            }
            for id in skipped {
                println!("S {} {} {}", note_id, link_type, id);
            }
        }
    }
}
