use clap::Subcommand;

/// Ontology subcommands
#[derive(Subcommand, Debug)]
pub enum OntologyCommands {
    /// Show the active ontology configuration
    Show {},
}
