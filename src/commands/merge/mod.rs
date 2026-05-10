//! Merge two notes into one
use crate::cli::{Cli, OutputFormat};
use qipu_core::error::{QipuError, Result};
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;
use qipu_core::note::TypedLink;
use qipu_core::store::Store;
use std::collections::HashSet;

pub fn execute(cli: &Cli, store: &Store, id1: &str, id2: &str, dry_run: bool) -> Result<()> {
    if id1 == id2 {
        return Err(QipuError::UsageError(
            "Use: qipu merge <source-id> <target-id>\nSource and target must be different notes.\nRun `qipu merge --help` for full and advanced details."
                .to_string(),
        ));
    }

    let note1 = store
        .get_note(id1)
        .map_err(|e| merge_note_lookup_error(e, id1, id2))?;
    let mut note2 = store
        .get_note(id2)
        .map_err(|e| merge_note_lookup_error(e, id1, id2))?;

    let final_tags = merge_tags(&note1, &note2);
    let links = merge_links(&note1, &note2, id2);
    let new_body = merge_bodies(&note1, &note2, id1);

    if dry_run {
        output_dry_run(cli, id1, id2, &final_tags, links.len())?;
        return Ok(());
    }

    if cli.format == OutputFormat::Human && !cli.quiet {
        print_merge_message(id1, id2, dry_run);
    }

    let updated_inbound = redirect_inbound_links(cli, store, id1, id2)?;

    note2.frontmatter.tags = final_tags;
    note2.frontmatter.links = links;
    note2.body = new_body;
    store.save_note(&mut note2)?;
    store.delete_note(id1)?;

    output_merge_complete(cli, id1, id2, &updated_inbound)?;
    Ok(())
}

fn merge_note_lookup_error(error: QipuError, source: &str, target: &str) -> QipuError {
    match error {
        QipuError::NoteNotFound { .. } => QipuError::UsageError(format!(
            "{}\n\nUse: qipu merge {} {}\nFind note IDs with: qipu list\nRun `qipu merge --help` for full and advanced details.",
            error, source, target
        )),
        other => other,
    }
}

fn print_merge_message(id1: &str, id2: &str, dry_run: bool) {
    if !dry_run {
        println!("Merging {} into {}...", id1, id2);
    } else {
        println!("Dry run: would merge {} into {}", id1, id2);
    }
}

fn merge_tags(note1: &Note, note2: &Note) -> Vec<String> {
    let mut tags: HashSet<String> = note2.frontmatter.tags.iter().cloned().collect();
    tags.extend(note1.frontmatter.tags.iter().cloned());
    let mut final_tags: Vec<String> = tags.into_iter().collect();
    final_tags.sort();
    final_tags
}

fn merge_links(note1: &Note, note2: &Note, target_id: &str) -> Vec<TypedLink> {
    let mut links: Vec<TypedLink> = note2.frontmatter.links.clone();
    let existing_link_ids: HashSet<(String, String)> = links
        .iter()
        .map(|l| (l.link_type.to_string(), l.id.clone()))
        .collect();

    for link in &note1.frontmatter.links {
        if link.id != target_id
            && !existing_link_ids.contains(&(link.link_type.to_string(), link.id.clone()))
        {
            links.push(link.clone());
        }
    }
    links
}

fn merge_bodies(note1: &Note, note2: &Note, source_id: &str) -> String {
    format!(
        "{}\n\n---\n\n### Merged from {}\n\n{}",
        note2.body.trim(),
        source_id,
        note1.body.trim()
    )
}

fn output_dry_run(
    cli: &Cli,
    source: &str,
    target: &str,
    tags: &[String],
    links_count: usize,
) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "dry_run": true,
                "source": source,
                "target": target,
                "tags": tags,
                "links_count": links_count,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!("Dry run: would merge {} into {}", source, target);
                println!("Tags would be: {:?}", tags);
                println!("Links count would be: {}", links_count);
            }
        }
        OutputFormat::Records => {
            println!(
                "M source={} target={} dry_run=true links={} tags={}",
                source,
                target,
                links_count,
                tags.join(",")
            );
        }
    }
    Ok(())
}

fn output_merge_complete(
    cli: &Cli,
    source: &str,
    target: &str,
    updated_inbound: &[String],
) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "merged": true,
                "source": source,
                "target": target,
                "updated_inbound": updated_inbound,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Merge complete. {} has been merged into {}.",
                    source, target
                );
            }
        }
        OutputFormat::Records => {
            println!(
                "M source={} target={} merged=true updated_inbound={}",
                source,
                target,
                updated_inbound.len()
            );
        }
    }
    Ok(())
}

fn redirect_inbound_links(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
) -> Result<Vec<String>> {
    let index = IndexBuilder::new(store).build()?;
    let inbound = index.get_inbound_edges(from_id);
    let source_ids: HashSet<String> = inbound.iter().map(|e| e.from.clone()).collect();
    let mut updated = Vec::new();

    for source_id in source_ids {
        if source_id == from_id {
            continue;
        }

        let mut source_note = store.get_note(&source_id)?;
        let mut modified = false;

        for link in &mut source_note.frontmatter.links {
            if link.id == from_id {
                link.id = to_id.to_string();
                modified = true;
            }
        }

        let mut seen_links = HashSet::new();
        source_note
            .frontmatter
            .links
            .retain(|l| seen_links.insert((l.link_type.to_string(), l.id.clone())));

        let old_link = format!("[[{}]]", from_id);
        let new_link = format!("[[{}]]", to_id);
        if source_note.body.contains(&old_link) {
            source_note.body = source_note.body.replace(&old_link, &new_link);
            modified = true;
        }

        let old_piped_prefix = format!("[[{}|", from_id);
        let new_piped_prefix = format!("[[{}|", to_id);
        if source_note.body.contains(&old_piped_prefix) {
            source_note.body = source_note
                .body
                .replace(&old_piped_prefix, &new_piped_prefix);
            modified = true;
        }

        if modified {
            if cli.format == OutputFormat::Human && !cli.quiet {
                println!("Updating inbound links in {}...", source_id);
            }
            store.save_note(&mut source_note)?;
            updated.push(source_id);
        }
    }

    updated.sort();
    Ok(updated)
}
