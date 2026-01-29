//! Link remove command
use crate::cli::{Cli, OutputFormat};
use qipu_core::error::Result;
use qipu_core::note::LinkType;
use qipu_core::store::Store;

use super::resolve_note_id;

/// Execute the link remove command
///
/// Removes a typed link from one note to another.
pub fn execute(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    link_type: LinkType,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load the source note
    let mut from_note = store.get_note(&from_resolved)?;

    // Find and remove the link
    let original_len = from_note.frontmatter.links.len();
    from_note
        .frontmatter
        .links
        .retain(|l| !(l.id == to_resolved && l.link_type == link_type));

    if from_note.frontmatter.links.len() == original_len {
        // Link didn't exist
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "not_found",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link does not exist"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} does not exist",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!(
                        "H qipu=1 records=1 store={} mode=link.remove status=not_found",
                        store.root().display()
                    );
                }
            }
        }
        return Ok(());
    }

    // Save the note
    store.save_note(&mut from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "removed",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Removed link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=link.remove status=removed",
                store.root().display()
            );
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}
