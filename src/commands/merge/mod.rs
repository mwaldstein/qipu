//! Merge two notes into one
use crate::cli::Cli;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::IndexBuilder;
use crate::lib::note::TypedLink;
use crate::lib::store::Store;
use std::collections::HashSet;
use std::fs;

/// Execute the merge command
pub fn execute(_cli: &Cli, store: &Store, id1: &str, id2: &str, dry_run: bool) -> Result<()> {
    if id1 == id2 {
        return Err(QipuError::Other(
            "cannot merge a note into itself".to_string(),
        ));
    }

    // 1. Load both notes
    let note1 = store.get_note(id1)?;
    let mut note2 = store.get_note(id2)?;

    if !dry_run {
        println!("Merging {} into {}...", id1, id2);
    } else {
        println!("Dry run: would merge {} into {}", id1, id2);
    }

    // 2. Combine Tags
    let mut tags: HashSet<String> = note2.frontmatter.tags.iter().cloned().collect();
    for tag in &note1.frontmatter.tags {
        tags.insert(tag.clone());
    }
    let mut final_tags: Vec<String> = tags.into_iter().collect();
    final_tags.sort();

    // 3. Combine Typed Links
    let mut links: Vec<TypedLink> = note2.frontmatter.links.clone();
    let existing_link_ids: HashSet<(String, String)> = links
        .iter()
        .map(|l| (l.link_type.to_string(), l.id.clone()))
        .collect();

    for link in &note1.frontmatter.links {
        // Skip links to id2 (they will become self-links) and duplicates
        if link.id != id2
            && !existing_link_ids.contains(&(link.link_type.to_string(), link.id.clone()))
        {
            links.push(link.clone());
        }
    }

    // 4. Combine Body
    let new_body = format!(
        "{}\n\n---\n\n### Merged from {}\n\n{}",
        note2.body.trim(),
        id1,
        note1.body.trim()
    );

    if dry_run {
        println!("Tags would be: {:?}", final_tags);
        println!("Links count would be: {}", links.len());
        return Ok(());
    }

    // 5. Update Inbound Links
    let index = IndexBuilder::new(store).build()?;
    let inbound = index.get_inbound_edges(id1);

    // Find unique source notes that link to id1
    let source_ids: HashSet<String> = inbound.iter().map(|e| e.from.clone()).collect();

    for source_id in source_ids {
        if source_id == id1 {
            continue;
        } // Skip self-links in note1

        let mut source_note = store.get_note(&source_id)?;
        let mut modified = false;

        // Update typed links in frontmatter
        for link in &mut source_note.frontmatter.links {
            if link.id == id1 {
                link.id = id2.to_string();
                modified = true;
            }
        }

        // Deduplicate links in case source_note already linked to id2 with same type
        let mut seen_links = HashSet::new();
        source_note
            .frontmatter
            .links
            .retain(|l| seen_links.insert((l.link_type.to_string(), l.id.clone())));

        // Update wiki-links in body [[id1]] -> [[id2]]
        // Simple replacement for now, could be improved with regex
        let old_link = format!("[[{}]]", id1);
        let new_link = format!("[[{}]]", id2);
        if source_note.body.contains(&old_link) {
            source_note.body = source_note.body.replace(&old_link, &new_link);
            modified = true;
        }

        // Also handle piped links [[id1|label]]
        let old_piped_prefix = format!("[[{}|", id1);
        let new_piped_prefix = format!("[[{}|", id2);
        if source_note.body.contains(&old_piped_prefix) {
            source_note.body = source_note
                .body
                .replace(&old_piped_prefix, &new_piped_prefix);
            modified = true;
        }

        if modified {
            println!("Updating inbound links in {}...", source_id);
            store.save_note(&mut source_note)?;
        }
    }

    // 6. Save Target Note
    note2.frontmatter.tags = final_tags;
    note2.frontmatter.links = links;
    note2.body = new_body;
    store.save_note(&mut note2)?;

    // 7. Delete Source Note
    if let Some(path) = &note1.path {
        fs::remove_file(path)?;
    }

    println!("Merge complete. {} has been merged into {}.", id1, id2);

    Ok(())
}
