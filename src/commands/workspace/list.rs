use crate::cli::Cli;
use crate::cli::OutputFormat;
use crate::lib::error::Result;
use crate::lib::store::paths::{WORKSPACES_DIR, WORKSPACE_FILE};
use crate::lib::store::workspace::WorkspaceMetadata;
use crate::lib::store::Store;
use std::env;
use std::path::PathBuf;
use std::time::Instant;
use tracing::debug;

pub fn execute(cli: &Cli) -> Result<()> {
    let start = Instant::now();
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if cli.verbose {
        debug!(root = %root.display(), "list_root");
    }

    let primary_store = Store::discover(&root)?;

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discovered_primary");
    }
    let workspaces_dir = primary_store.root().join(WORKSPACES_DIR);

    let mut workspaces = Vec::new();

    // Add primary
    workspaces.push(WorkspaceInfo {
        name: "(primary)".to_string(),
        path: primary_store.root().to_path_buf(),
        temporary: false,
        note_count: primary_store.list_notes()?.len(),
    });

    if workspaces_dir.is_dir() {
        if cli.verbose {
            debug!("discovering_workspaces");
        }
        for entry in std::fs::read_dir(workspaces_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(store) = Store::open(&path) {
                    let metadata = WorkspaceMetadata::load(&path.join(WORKSPACE_FILE)).ok();
                    workspaces.push(WorkspaceInfo {
                        name: metadata
                            .as_ref()
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| {
                                path.file_name().unwrap().to_string_lossy().into_owned()
                            }),
                        path: path.to_path_buf(),
                        temporary: metadata.map(|m| m.temporary).unwrap_or(false),
                        note_count: store.list_notes()?.len(),
                    });
                }
            }
        }
    }

    if cli.verbose {
        debug!(count = workspaces.len(), elapsed = ?start.elapsed(), "list_complete");
    }

    match cli.format {
        OutputFormat::Human => {
            println!(
                "{:<20} {:<10} {:<10} {:<20}",
                "Name", "Temp", "Notes", "Path"
            );
            println!("{}", "-".repeat(60));
            for ws in workspaces {
                println!(
                    "{:<20} {:<10} {:<10} {:<20}",
                    ws.name,
                    if ws.temporary { "Yes" } else { "No" },
                    ws.note_count,
                    ws.path.display()
                );
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&workspaces)?;
            println!("{}", json);
        }
        OutputFormat::Records => {
            for ws in workspaces {
                println!(
                    "WS {} temp={} notes={} path={}",
                    ws.name,
                    ws.temporary,
                    ws.note_count,
                    ws.path.display()
                );
            }
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct WorkspaceInfo {
    name: String,
    path: PathBuf,
    temporary: bool,
    note_count: usize,
}
