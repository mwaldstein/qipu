use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::store::paths::{WORKSPACES_DIR, WORKSPACE_FILE};
use crate::lib::store::workspace::WorkspaceMetadata;
use crate::lib::store::Store;
use std::env;
use std::path::PathBuf;

pub fn execute(
    cli: &Cli,
    name: &str,
    temp: bool,
    _empty: bool,
    copy_primary: bool,
    from_tag: Option<&str>,
    from_note: Option<&str>,
    from_query: Option<&str>,
) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let primary_store = Store::discover(&root)?;
    let workspace_path = primary_store.root().join(WORKSPACES_DIR).join(name);

    if workspace_path.exists() {
        return Err(crate::lib::error::QipuError::Other(format!(
            "workspace '{}' already exists",
            name
        )));
    }

    // Initialize the new store
    let options = crate::lib::store::InitOptions {
        visible: false,
        stealth: false,
        branch: None,
    };
    let ws_store = Store::init_at(&workspace_path, options, None)?;

    // Save workspace metadata
    let metadata = WorkspaceMetadata {
        name: name.to_string(),
        created_at: chrono::Utc::now(),
        temporary: temp,
        parent_id: None, // Could set to primary ID if we had one
    };
    metadata.save(&workspace_path.join(WORKSPACE_FILE))?;

    if copy_primary {
        copy_notes(&primary_store, &ws_store)?;
    } else if let Some(tag) = from_tag {
        let notes = primary_store.list_notes()?;
        for note in notes {
            if note.frontmatter.tags.contains(&tag.to_string()) {
                copy_note(&note, &ws_store)?;
            }
        }
    } else if let Some(note_id) = from_note {
        // This should be a graph slice, but for now just copy the note
        let note = primary_store.get_note(note_id)?;
        copy_note(&note, &ws_store)?;
    } else if let Some(query) = from_query {
        // Simple search and copy
        let notes = primary_store.list_notes()?;
        for note in notes {
            if note.title().contains(query) || note.body.contains(query) {
                copy_note(&note, &ws_store)?;
            }
        }
    }

    if !cli.quiet {
        println!(
            "Created workspace '{}' at {}",
            name,
            workspace_path.display()
        );
    }

    Ok(())
}

fn copy_notes(src: &Store, dst: &Store) -> Result<()> {
    for note in src.list_notes()? {
        copy_note(&note, dst)?;
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
