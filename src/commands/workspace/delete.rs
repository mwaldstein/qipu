use crate::Cli;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::fs;
use std::path::Path;

pub fn execute(cli: &Cli, root: &Path, name: &str, force: bool) -> Result<()> {
    let primary_store = Store::discover(root)?;
    let workspace_path = primary_store.workspaces_dir().join(name);

    if !workspace_path.exists() {
        return Err(qipu_core::error::QipuError::not_found("workspace", name));
    }

    if !force {
        // Try to open the workspace as a store to check for unmerged changes
        if let Ok(workspace_store) = Store::open(&workspace_path) {
            let changes = check_unmerged_changes(&workspace_store, &primary_store)?;
            if !changes.is_empty() {
                tracing::warn!("Workspace '{}' has unmerged changes:", name);
                for change in changes {
                    tracing::warn!("  - {}", change);
                }
                return Err(qipu_core::error::QipuError::UsageError(
                    "Use --force to delete anyway".to_string(),
                ));
            }
        }
    }

    fs::remove_dir_all(&workspace_path)?;

    if !cli.quiet {
        println!("Deleted workspace '{}'", name);
    }

    Ok(())
}

fn check_unmerged_changes(workspace: &Store, primary: &Store) -> Result<Vec<String>> {
    let mut changes = Vec::new();
    let notes = workspace.list_notes()?;

    for note in notes {
        match primary.get_note(note.id()) {
            Ok(primary_note) => {
                if is_modified(&note, &primary_note)? {
                    changes.push(format!("Modified: {} ({})", note.title(), note.id()));
                }
            }
            Err(_) => {
                changes.push(format!("New: {} ({})", note.title(), note.id()));
            }
        }
    }

    Ok(changes)
}

fn is_modified(n1: &Note, n2: &Note) -> Result<bool> {
    // 1. Compare body content
    if n1.body != n2.body {
        return Ok(true);
    }

    // 2. Compare frontmatter (excluding 'updated' field)
    let mut v1 = serde_json::to_value(&n1.frontmatter)?;
    let mut v2 = serde_json::to_value(&n2.frontmatter)?;

    if let Some(obj) = v1.as_object_mut() {
        obj.remove("updated");
    }
    if let Some(obj) = v2.as_object_mut() {
        obj.remove("updated");
    }

    Ok(v1 != v2)
}
