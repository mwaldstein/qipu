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

    let mut conflicts: Vec<(String, &str)> = Vec::new();
    let mut additions: Vec<String> = Vec::new();

    for note in &source_notes {
        let id: String = note.id().to_string();
        if target_notes_ids.contains(&id) {
            let action = match strategy {
                "overwrite" => "overwrite",
                "merge-links" => "merge-links",
                "skip" | _ => "skip",
            };
            conflicts.push((id.clone(), action));
            if !dry_run {
                match strategy {
                    "overwrite" => {
                        copy_note(note, &target_store)?;
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
                    "skip" | _ => {}
                }
            }
        } else {
            additions.push(id.clone());
            if !dry_run {
                copy_note(note, &target_store)?;
            }
        }
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

    if delete_source && source_name != "." {
        std::fs::remove_dir_all(source_store.root())?;
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

fn copy_note(note: &crate::lib::note::Note, dst: &Store) -> Result<()> {
    let mut new_note = note.clone();

    // Determine target directory
    let target_dir = match new_note.note_type() {
        crate::lib::note::NoteType::Moc => dst.mocs_dir(),
        _ => dst.notes_dir(),
    };

    // Determine file path
    let id_obj = crate::lib::id::NoteId::new_unchecked(new_note.id().to_string());
    let file_name = crate::lib::id::filename(&id_obj, new_note.title());
    let file_path = target_dir.join(&file_name);

    new_note.path = Some(file_path);

    dst.save_note(&mut new_note)?;
    Ok(())
}
