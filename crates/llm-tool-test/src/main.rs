mod adapter;
mod cli;
mod fixture;
mod scenario;
mod session;
mod transcript;

use adapter::{amp::AmpAdapter, opencode::OpenCodeAdapter, ToolAdapter};
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run {
            scenario,
            tags,
            tool,
            max_usd,
            dry_run,
        } => {
            println!(
                "Run command: scenario={:?}, tags={:?}, tool={}, max_usd={:?}, dry_run={}",
                scenario, tags, tool, max_usd, dry_run
            );

            let adapter: Box<dyn ToolAdapter> = match tool.as_str() {
                "amp" => Box::new(AmpAdapter),
                "opencode" => Box::new(OpenCodeAdapter),
                _ => anyhow::bail!("Unknown tool: {}", tool),
            };

            if let Some(path) = scenario {
                let s = scenario::load(path)?;
                println!("Loaded scenario: {}", s.name);

                if !*dry_run {
                    println!("Checking availability for tool: {}", tool);
                    if let Err(e) = adapter.check_availability() {
                        eprintln!("Warning: Tool '{}' check failed: {}", tool, e);
                        // We might want to continue if it's just a warning or fail hard?
                        // For now, let's fail hard as per plan "Graceful skip if tool unavailable"
                        // But if user explicitly requested it, maybe fail?
                        // The plan says "Graceful skip if tool unavailable" under "Availability checks"
                        // but here we are running a specific tool.
                        anyhow::bail!("Tool unavailable: {}", e);
                    }

                    println!("Setting up environment for fixture: {}", s.fixture);
                    let env = fixture::TestEnv::new(&s.name)?;
                    env.setup_fixture(&s.fixture)?;
                    println!("Environment created at: {:?}", env.root);

                    println!("Running tool '{}'...", tool);
                    let output = adapter.run(&s, &env.root)?;
                    println!("Tool finished.");

                    // Transcript test
                    let writer = transcript::TranscriptWriter::new(env.root.join("artifacts"))?;
                    writer.write_raw(&output)?;
                    writer.append_event(&serde_json::json!({
                        "type": "execution",
                        "tool": tool,
                        "output": output
                    }))?;
                    println!("Transcript written to artifacts/");
                }
            }
        }
        Commands::List { tags } => {
            println!("List command: tags={:?}", tags);
        }
        Commands::Show { name } => {
            println!("Show command: name={}", name);
        }
        Commands::Compare { run_ids } => {
            println!("Compare command: run_ids={:?}", run_ids);
        }
        Commands::Clean => {
            println!("Clean command");
        }
    }
    Ok(())
}
