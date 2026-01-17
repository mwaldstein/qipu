use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::store::paths::WORKSPACES_DIR;
use crate::lib::store::Store;
use std::env;
use std::path::PathBuf;

pub fn execute(
    cli: &Cli,
    source_name: &str,
    target_name: &str,
    dry_run: bool,
    strategy: &str,
    delete_source: bool,
) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let primary_store = Store::discover(&root)?;

    let source_store = if source_name == "." {
        Store::discover(&root)?
    } else {
        let path = primary_store.root().join(WORKSPACES_DIR).join(source_name);
        Store::open(&path)?
    };

    let target_store = if target_name == "." {
        Store::discover(&root)?
    } else {
        let path = primary_store.root().join(WORKSPACES_DIR).join(target_name);
        Store::open(&path)?
    };

    let source_notes = source_store.list_notes()?;
    let target_notes_ids = target_store.existing_ids()?;

    for note in source_notes {
        let id: String = note.id().to_string();
        if target_notes_ids.contains(&id) {
            match strategy {
                "overwrite" => {
                    if !dry_run {
                        target_store.create_note_with_content(
                            note.title(),
                            Some(note.note_type()),
                            &note.frontmatter.tags,
                            &note.body,
                        )?;
                    }
                }
                "merge-links" => {
                    if !dry_run {
                        let mut target_note = target_store.get_note(&id)?;
                        // Simple merge: union of tags (as a proxy for links for now)
                        for tag in &note.frontmatter.tags {
                            if !target_note.frontmatter.tags.contains(tag) {
                                target_note.frontmatter.tags.push(tag.clone());
                            }
                        }
                        target_store.save_note(&mut target_note)?;
                    }
                }
                "skip" | _ => {
                    // Default skip
                }
            }
        } else {
            if !dry_run {
                copy_note(&note, &target_store)?;
            }
        }
    }

    if delete_source && !dry_run && source_name != "." {
        std::fs::remove_dir_all(source_store.root())?;
    }

    if !cli.quiet {
        println!("Merged workspace '{}' into '{}'", source_name, target_name);
    }

    Ok(())
}

fn copy_note(note: &crate::lib::note::Note, dst: &Store) -> Result<()> {
    dst.create_note_with_content(
        note.title(),
        Some(note.note_type()),
        &note.frontmatter.tags,
        &note.body,
    )?;
    Ok(())
}
