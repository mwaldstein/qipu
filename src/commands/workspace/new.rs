use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::index::Index;
use crate::lib::index::IndexBuilder;
use crate::lib::store::config;
use crate::lib::store::paths::{GITIGNORE_FILE, WORKSPACES_DIR, WORKSPACE_FILE};
use crate::lib::store::workspace::WorkspaceMetadata;
use crate::lib::store::Store;
use std::collections::{HashSet, VecDeque};
use std::env;
use std::path::PathBuf;
use std::time::Instant;
use tracing::debug;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    name: &str,
    temp: bool,
    empty: bool,
    copy_primary: bool,
    from_tag: Option<&str>,
    from_note: Option<&str>,
    from_query: Option<&str>,
) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        debug!(name, temp, empty, copy_primary, "new_params");
    }

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
        no_index: true,
        index_strategy: None,
    };
    let ws_store = Store::init_at(&workspace_path, options, None)?;

    if cli.verbose {
        debug!(path = %workspace_path.display(), "workspace_initialized");
    }

    // Save workspace metadata
    // All workspaces are created from the primary store, so parent_id is "(primary)"
    let metadata = WorkspaceMetadata {
        name: name.to_string(),
        created_at: chrono::Utc::now(),
        temporary: temp,
        parent_id: Some("(primary)".to_string()),
    };
    metadata.save(&workspace_path.join(WORKSPACE_FILE))?;

    if temp {
        if let Some(project_root) = primary_store.root().parent() {
            let project_gitignore = project_root.join(GITIGNORE_FILE);
            let workspace_relative_path = workspace_path
                .strip_prefix(project_root)
                .map(|p| format!("{}/", p.display()))
                .unwrap_or_else(|_| format!("{}/{}/{}/", ".qipu", WORKSPACES_DIR, name));
            config::ensure_project_gitignore_entry(&project_gitignore, &workspace_relative_path)?;
        }
    }

    if !empty {
        if cli.verbose {
            debug!("copying_notes_to_workspace");
        }
        if copy_primary {
            copy_notes(&primary_store, &ws_store)?;
        } else if let Some(tag) = from_tag {
            // Collect all notes matching the tag, then perform graph slice from them
            let notes = primary_store.list_notes()?;
            let root_ids: Vec<String> = notes
                .iter()
                .filter(|note| note.frontmatter.tags.contains(&tag.to_string()))
                .map(|note| note.id().to_string())
                .collect();

            if !root_ids.is_empty() {
                let index = IndexBuilder::new(&primary_store).build()?;
                copy_graph_slice(&primary_store, &index, &root_ids, &ws_store)?;
            }
        } else if let Some(note_id) = from_note {
            let index = IndexBuilder::new(&primary_store).build()?;
            copy_graph_slice(&primary_store, &index, &[note_id.to_string()], &ws_store)?;
        } else if let Some(query) = from_query {
            // Collect all notes matching the query, then perform graph slice from them
            let notes = primary_store.list_notes()?;
            let root_ids: Vec<String> = notes
                .iter()
                .filter(|note| note.title().contains(query) || note.body.contains(query))
                .map(|note| note.id().to_string())
                .collect();

            if !root_ids.is_empty() {
                let index = IndexBuilder::new(&primary_store).build()?;
                copy_graph_slice(&primary_store, &index, &root_ids, &ws_store)?;
            }
        }
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "workspace_created");
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

fn copy_graph_slice(src: &Store, index: &Index, root_ids: &[String], dst: &Store) -> Result<()> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();

    // Initialize queue with all root IDs
    for root_id in root_ids {
        queue.push_back((root_id.to_string(), 0));
        visited.insert(root_id.to_string());
    }

    while let Some((current_id, hops)) = queue.pop_front() {
        if hops >= 3 {
            continue;
        }

        // Copy the current note
        match src.get_note(&current_id) {
            Ok(note) => {
                copy_note(&note, dst)?;
            }
            Err(_) => {
                return Err(crate::lib::error::QipuError::NoteNotFound {
                    id: current_id.to_string(),
                });
            }
        }

        // Explore neighbors
        for edge in &index.edges {
            let neighbor_id = if edge.from == current_id {
                Some(&edge.to)
            } else if edge.to == current_id {
                Some(&edge.from)
            } else {
                None
            };

            if let Some(nid) = neighbor_id {
                if !visited.contains(nid) {
                    visited.insert(nid.to_string());
                    queue.push_back((nid.clone(), hops + 1));
                }
            }
        }
    }

    Ok(())
}
