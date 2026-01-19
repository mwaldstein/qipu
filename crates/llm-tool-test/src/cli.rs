use clap::{Parser, Subcommand};

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

        /// Tool to test (e.g., amp, claude-code, opencode)
        #[arg(long, default_value = "opencode")]
        tool: String,

        /// Model to use with the tool (e.g., claude-sonnet-4-20250514, gpt-4o)
        #[arg(long)]
        model: Option<String>,

        /// Multiple tools for matrix run (comma-separated, e.g., opencode,amp)
        #[arg(long)]
        tools: Option<String>,

        /// Multiple models for matrix run (comma-separated, e.g., claude-sonnet-4-20250514,gpt-4o)
        #[arg(long)]
        models: Option<String>,

        /// Maximum cost in USD
        #[arg(long)]
        max_usd: Option<f64>,

        /// Dry run (don't execute LLM calls)
        #[arg(long)]
        dry_run: bool,

        /// Disable caching
        #[arg(long)]
        no_cache: bool,

        /// Judge model for LLM-as-judge evaluation
        #[arg(long)]
        judge_model: Option<String>,
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
