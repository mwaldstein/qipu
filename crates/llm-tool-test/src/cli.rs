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

        /// Run all scenarios in fixtures directory
        #[arg(long)]
        all: bool,

        /// Filter scenarios by tags
        #[arg(long)]
        tags: Vec<String>,

        /// Filter scenarios by tier (0=smoke, 1=quick, 2=standard, 3=comprehensive)
        #[arg(long, default_value = "0")]
        tier: usize,

        /// Tool to test (e.g., claude-code, opencode)
        #[arg(long, default_value = "opencode")]
        tool: String,

        /// Model to use with the tool (e.g., claude-sonnet-4-20250514, gpt-4o)
        #[arg(long)]
        model: Option<String>,

        /// Multiple tools for matrix run (comma-separated, e.g., opencode,claude-code)
        #[arg(long)]
        tools: Option<String>,

        /// Multiple models for matrix run (comma-separated, e.g., claude-sonnet-4-20250514,gpt-4o)
        #[arg(long)]
        models: Option<String>,

        /// Dry run (don't execute LLM calls)
        #[arg(long)]
        dry_run: bool,

        /// Disable caching
        #[arg(long)]
        no_cache: bool,

        /// Judge model for LLM-as-judge evaluation
        #[arg(long)]
        judge_model: Option<String>,

        /// Disable LLM-as-judge evaluation
        #[arg(long)]
        no_judge: bool,

        /// Maximum execution time in seconds per command
        #[arg(long, default_value = "300")]
        timeout_secs: u64,

        /// Maximum budget in USD for this session (overrides env var)
        #[arg(long)]
        max_usd: Option<f64>,
    },
    /// List available scenarios
    Scenarios {
        /// Filter by tags
        #[arg(long)]
        tags: Vec<String>,

        /// Filter scenarios by tier (0=smoke, 1=quick, 2=standard, 3=comprehensive)
        #[arg(long, default_value = "0")]
        tier: usize,
    },
    /// Show details of a scenario
    Show {
        /// Name of the scenario
        #[arg(required = true)]
        name: String,
    },
    /// Clean up artifacts
    Clean {
        /// Clean artifacts older than duration (e.g., "30d", "7d", "1h")
        #[arg(long)]
        older_than: Option<String>,
    },
}
