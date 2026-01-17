use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a test scenario
    Run {
        /// Path to scenario file or name
        #[arg(long, short)]
        scenario: Option<String>,

        /// Filter scenarios by tags
        #[arg(long)]
        tags: Vec<String>,

        /// Tool to test (e.g., qipu)
        #[arg(long, default_value = "qipu")]
        tool: String,

        /// Maximum cost in USD
        #[arg(long)]
        max_usd: Option<f64>,

        /// Dry run (don't execute LLM calls)
        #[arg(long)]
        dry_run: bool,
    },
    /// List available scenarios
    List {
        /// Filter by tags
        #[arg(long)]
        tags: Vec<String>,
    },
    /// Show details of a scenario
    Show {
        /// Name of the scenario
        #[arg(required = true)]
        name: String,
    },
    /// Compare results of runs
    Compare {
        /// Run IDs to compare
        #[arg(required = true)]
        run_ids: Vec<String>,
    },
    /// Clean up artifacts
    Clean,
}
