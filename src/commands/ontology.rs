//! `qipu ontology` commands - manage and display ontology configuration

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::ontology::Ontology;
use crate::lib::store::Store;

/// Execute the show command
pub fn execute_show(cli: &Cli, store: &Store) -> Result<()> {
    let config = store.config();
    let ontology = Ontology::from_config_with_graph(&config.ontology, &config.graph);
    let mode = config.ontology.mode;

    let note_types = ontology.note_types();
    let link_types = ontology.link_types();

    match cli.format {
        OutputFormat::Json => {
            let note_type_objs: Vec<_> = note_types
                .iter()
                .map(|nt| {
                    let config = config.ontology.note_types.get(nt);
                    serde_json::json!({
                        "name": nt,
                        "description": config.and_then(|c| c.description.clone()),
                    })
                })
                .collect();

            let link_type_objs: Vec<_> = link_types
                .iter()
                .map(|lt| {
                    let inverse = ontology.get_inverse(lt);
                    let config = config.ontology.link_types.get(lt);
                    serde_json::json!({
                        "name": lt,
                        "inverse": inverse,
                        "description": config.and_then(|c| c.description.clone()),
                    })
                })
                .collect();

            let output = serde_json::json!({
                "mode": serde_json::to_value(mode)?,
                "note_types": note_type_objs,
                "link_types": link_type_objs,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            println!("Ontology mode: {}", format_mode(mode));
            println!();

            if note_types.is_empty() {
                println!("No note types defined");
            } else {
                println!("Note types:");
                for nt in &note_types {
                    if let Some(config) = config.ontology.note_types.get(nt) {
                        if let Some(desc) = &config.description {
                            println!("  {}: {}", nt, desc);
                        } else {
                            println!("  {}", nt);
                        }
                    } else {
                        println!("  {}", nt);
                    }
                }
            }
            println!();

            if link_types.is_empty() {
                println!("No link types defined");
            } else {
                println!("Link types:");
                for lt in &link_types {
                    let inverse = ontology.get_inverse(lt);
                    if let Some(config) = config.ontology.link_types.get(lt) {
                        if let Some(desc) = &config.description {
                            println!("  {} -> {} ({})", lt, inverse, desc);
                        } else {
                            println!("  {} -> {}", lt, inverse);
                        }
                    } else {
                        println!("  {} -> {}", lt, inverse);
                    }
                }
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=ontology.show",
                store.root().display()
            );
            println!("O mode={}", format_mode(mode).to_lowercase());

            for nt in &note_types {
                if let Some(config) = config.ontology.note_types.get(nt) {
                    if let Some(desc) = &config.description {
                        println!("N note_type=\"{}\" description=\"{}\"", nt, desc);
                    } else {
                        println!("N note_type=\"{}\"", nt);
                    }
                } else {
                    println!("N note_type=\"{}\"", nt);
                }
            }

            for lt in &link_types {
                let inverse = ontology.get_inverse(lt);
                if let Some(config) = config.ontology.link_types.get(lt) {
                    if let Some(desc) = &config.description {
                        println!(
                            "L link_type=\"{}\" inverse=\"{}\" description=\"{}\"",
                            lt, inverse, desc
                        );
                    } else {
                        println!("L link_type=\"{}\" inverse=\"{}\"", lt, inverse);
                    }
                } else {
                    println!("L link_type=\"{}\" inverse=\"{}\"", lt, inverse);
                }
            }
        }
    }

    Ok(())
}

fn format_mode(mode: crate::lib::config::OntologyMode) -> &'static str {
    match mode {
        crate::lib::config::OntologyMode::Default => "default",
        crate::lib::config::OntologyMode::Extended => "extended",
        crate::lib::config::OntologyMode::Replacement => "replacement",
    }
}
