mod adapter;
mod cli;
mod commands;
mod evaluation;
mod fixture;
mod judge;
mod results;
mod run;
mod scenario;
mod session;
mod store_analysis;
mod transcript;

use clap::Parser;
use cli::Cli;
use commands::{print_result_summary, print_regression_report};
use results::{Cache, ResultsDB};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dir = std::path::PathBuf::from("target/llm_test_runs");
    let results_db = ResultsDB::new(&base_dir);
    let cache = Cache::new(&base_dir);

    match &cli.command {
        cli::Commands::Run { .. } => {
            commands::handle_run(&cli.command, &base_dir, &results_db, &cache)?;
        }
        cli::Commands::List { .. } => {
            commands::handle_list(&cli.command, &results_db)?;
        }
        cli::Commands::Show { .. } => {
            commands::handle_show(&cli.command, &results_db)?;
        }
        cli::Commands::Compare { .. } => {
            commands::handle_compare(&cli.command, &results_db)?;
        }
        cli::Commands::Clean => {
            commands::handle_clean(&cache)?;
        }
        cli::Commands::Review { .. } => {
            commands::handle_review(&cli.command, &results_db)?;
        }
    }
    Ok(())
}
