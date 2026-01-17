mod cli;
mod fixture;
mod scenario;
mod session;
mod transcript;

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

            if let Some(path) = scenario {
                let s = scenario::load(path)?;
                println!("Loaded scenario: {}", s.name);

                if !*dry_run {
                    println!("Setting up environment for fixture: {}", s.fixture);
                    let env = fixture::TestEnv::new(&s.name)?;
                    env.setup_fixture(&s.fixture)?;
                    println!("Environment created at: {:?}", env.root);

                    println!("Running 'ls' in environment...");
                    let runner = session::SessionRunner::new();
                    let output = runner.run_command("ls", &["-F"], &env.root)?;
                    println!("Output:\n{}", output);

                    // Transcript test
                    let writer = transcript::TranscriptWriter::new(env.root.join("artifacts"))?;
                    writer.write_raw(&output)?;
                    writer.append_event(&serde_json::json!({
                        "type": "command",
                        "cmd": "ls -F",
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
