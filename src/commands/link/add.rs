//! Link add command
use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::{LinkType, TypedLink};
use crate::lib::store::Store;

use super::resolve_note_id;

/// Execute the link add command
///
/// Adds a typed link from one note to another.
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

    // Load and verify both notes exist
    let mut from_note = store.get_note(&from_resolved)?;
    let _to_note = store.get_note(&to_resolved)?;

    // Check if link already exists
    let link_exists = from_note
        .frontmatter
        .links
        .iter()
        .any(|l| l.id == to_resolved && l.link_type == link_type);

    if link_exists {
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "unchanged",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link already exists"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} already exists",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!(
                        "H qipu=1 records=1 store={} mode=link.add status=unchanged",
                        store.root().display()
                    );
                    println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
                }
            }
        }
        return Ok(());
    }

    // Add the link
    from_note.frontmatter.links.push(TypedLink {
        link_type,
        id: to_resolved.clone(),
    });

    // Save the note
    store.save_note(&mut from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "added",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Added link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=link.add status=added",
                store.root().display()
            );
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}
