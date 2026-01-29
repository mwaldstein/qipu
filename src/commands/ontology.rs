//! `qipu ontology` commands - manage and display ontology configuration

use crate::cli::{Cli, OutputFormat};
use crate::commands::format::{print_ontology_human, print_ontology_json, print_ontology_records};
use qipu_core::error::Result;
use qipu_core::ontology::Ontology;
use qipu_core::store::Store;

/// Execute the show command
pub fn execute_show(cli: &Cli, store: &Store) -> Result<()> {
    let config = store.config();
    let ontology = Ontology::from_config_with_graph(&config.ontology, &config.graph);

    let note_types = ontology.note_types();
    let link_types = ontology.link_types();

    match cli.format {
        OutputFormat::Json => print_ontology_json(cli, store, &ontology, &note_types, &link_types)?,
        OutputFormat::Human => print_ontology_human(store, &ontology, &note_types, &link_types),
        OutputFormat::Records => print_ontology_records(store, &ontology, &note_types, &link_types),
    }

    Ok(())
}
