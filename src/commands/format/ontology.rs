//! Ontology output formatting helpers

use crate::cli::Cli;
use qipu_core::error::Result;
use qipu_core::ontology::Ontology;
use qipu_core::store::Store;

/// Print ontology in JSON format
pub fn print_ontology_json(
    _cli: &Cli,
    store: &Store,
    ontology: &Ontology,
    note_types: &[String],
    link_types: &[String],
) -> Result<()> {
    let config = store.config();
    let mode = config.ontology.mode;

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
    Ok(())
}

/// Print ontology in Human format
pub fn print_ontology_human(
    store: &Store,
    ontology: &Ontology,
    note_types: &[String],
    link_types: &[String],
) {
    let config = store.config();
    let mode = config.ontology.mode;

    println!("Ontology mode: {}", mode);
    println!();

    if note_types.is_empty() {
        println!("No note types defined");
    } else {
        println!("Note types:");
        for nt in note_types {
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
        for lt in link_types {
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

/// Print ontology in Records format
pub fn print_ontology_records(
    store: &Store,
    ontology: &Ontology,
    note_types: &[String],
    link_types: &[String],
) {
    let config = store.config();
    let mode = config.ontology.mode;

    println!(
        "H qipu=1 records=1 store={} mode=ontology.show",
        store.root().display()
    );
    println!("O mode={}", mode);

    for nt in note_types {
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

    for lt in link_types {
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
