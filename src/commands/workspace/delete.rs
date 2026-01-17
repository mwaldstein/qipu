use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::store::paths::WORKSPACES_DIR;
use crate::lib::store::Store;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn execute(cli: &Cli, name: &str, _force: bool) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let primary_store = Store::discover(&root)?;
    let workspace_path = primary_store.root().join(WORKSPACES_DIR).join(name);

    if !workspace_path.exists() {
        return Err(crate::lib::error::QipuError::Other(format!(
            "workspace '{}' not found",
            name
        )));
    }

    fs::remove_dir_all(&workspace_path)?;

    if !cli.quiet {
        println!("Deleted workspace '{}'", name);
    }

    Ok(())
}
