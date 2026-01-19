mod adapter;
mod cli;
mod evaluation;
mod fixture;
mod judge;
mod results;
mod scenario;
mod session;
mod transcript;

use adapter::{amp::AmpAdapter, opencode::OpenCodeAdapter, ToolAdapter};
use chrono::Utc;
use clap::Parser;
use cli::{Cli, Commands};
use results::{
    Cache, CacheKey, EfficiencyMetricsRecord, EvaluationMetricsRecord, GateResultRecord,
    RegressionReport, ResultRecord, ResultsDB,
};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dir = std::path::PathBuf::from("target/llm_test_runs");
    let results_db = ResultsDB::new(&base_dir);
    let cache = Cache::new(&base_dir);

    match &cli.command {
        Commands::Run {
            scenario,
            tags: _,
            tool,
            max_usd: _,
            dry_run,
            no_cache,
            judge_model,
        } => {
            if let Some(model) = judge_model {
                std::env::set_var("LLM_TOOL_TEST_JUDGE", model);
            }
            if let Some(path) = scenario {
                let s = scenario::load(path)?;
                println!("Loaded scenario: {}", s.name);

                let scenario_yaml = std::fs::read_to_string(path)?;
                let prompt = s.task.prompt.clone();

                let qipu_version = results::get_qipu_version()?;
                let cache_key = CacheKey::compute(&scenario_yaml, &prompt, tool, &qipu_version);

                if !no_cache {
                    if let Some(cached) = cache.get(&cache_key) {
                        println!("Cache HIT! Using cached result: {}", cached.id);
                        print_result_summary(&cached);
                        return Ok(());
                    }
                }

                let adapter: Box<dyn ToolAdapter> = match tool.as_str() {
                    "amp" => Box::new(AmpAdapter),
                    "opencode" => Box::new(OpenCodeAdapter),
                    _ => anyhow::bail!("Unknown tool: {}", tool),
                };

                if !*dry_run {
                    println!("Checking availability for tool: {}", tool);
                    if let Err(e) = adapter.check_availability() {
                        eprintln!("Warning: Tool '{}' check failed: {}", tool, e);
                        anyhow::bail!("Tool unavailable: {}", e);
                    }

                    println!("Setting up environment for fixture: {}", s.fixture);
                    let env = fixture::TestEnv::new(&s.name)?;
                    env.setup_fixture(&s.fixture)?;
                    println!("Environment created at: {:?}", env.root);

                    let start_time = Instant::now();
                    println!("Running tool '{}'...", tool);
                    let (output, exit_code) = adapter.run(&s, &env.root)?;
                    let duration = start_time.elapsed();

                    let transcript_dir = env.root.join("artifacts");
                    std::fs::create_dir_all(&transcript_dir)?;
                    let writer = transcript::TranscriptWriter::new(transcript_dir.clone())?;
                    writer.write_raw(&output)?;
                    writer.append_event(&serde_json::json!({
                        "type": "execution",
                        "tool": tool,
                        "output": output,
                        "exit_code": exit_code
                    }))?;

                    println!("Running evaluation...");
                    let metrics = evaluation::evaluate(&s, &env.root)?;
                    println!("Evaluation metrics: {:?}", metrics);

                    let outcome = if metrics.gates_passed < metrics.gates_total {
                        format!(
                            "Fail: {}/{} gates passed",
                            metrics.gates_passed, metrics.gates_total
                        )
                    } else {
                        "Pass".to_string()
                    };

                    let transcript_path = transcript_dir.to_string_lossy().to_string();

                    let record = ResultRecord {
                        id: results::generate_run_id(),
                        scenario_id: s.name.clone(),
                        scenario_hash: cache_key.scenario_hash.clone(),
                        tool: tool.clone(),
                        model: "default".to_string(),
                        qipu_commit: qipu_version.clone(),
                        timestamp: Utc::now(),
                        duration_secs: duration.as_secs_f64(),
                        cost_usd: 0.0,
                        gates_passed: metrics.gates_passed >= metrics.gates_total,
                        metrics: EvaluationMetricsRecord {
                            gates_passed: metrics.gates_passed,
                            gates_total: metrics.gates_total,
                            note_count: metrics.note_count,
                            link_count: metrics.link_count,
                            details: metrics
                                .details
                                .into_iter()
                                .map(|d| GateResultRecord {
                                    gate_type: d.gate_type,
                                    passed: d.passed,
                                    message: d.message,
                                })
                                .collect(),
                            efficiency: EfficiencyMetricsRecord {
                                total_commands: metrics.efficiency.total_commands,
                                unique_commands: metrics.efficiency.unique_commands,
                                error_count: metrics.efficiency.error_count,
                                retry_count: metrics.efficiency.retry_count,
                                help_invocations: metrics.efficiency.help_invocations,
                                first_try_success_rate: metrics.efficiency.first_try_success_rate,
                                iteration_ratio: metrics.efficiency.iteration_ratio,
                            },
                        },
                        judge_score: metrics.judge_score,
                        outcome,
                        transcript_path: transcript_path.clone(),
                        cache_key: Some(cache_key.as_string()),
                    };

                    results_db.append(&record)?;
                    cache.put(&cache_key, &record)?;

                    if let Some(baseline) = results_db.load_baseline(&s.name, tool)? {
                        let report = results::compare_runs(&record, &baseline);
                        print_regression_report(&report);
                    }

                    println!("\nRun completed: {}", record.id);
                    println!("Transcript written to: {}", transcript_path);
                    print_result_summary(&record);
                } else {
                    println!("Dry run - skipping execution");
                }
            } else {
                println!("No scenario specified. Use --scenario <path>");
            }
        }
        Commands::List { tags: _ } => {
            let records = results_db.load_all()?;
            println!("Recent runs: {}", records.len());
            for record in records.iter().rev().take(10) {
                println!(
                    "  {} - {} ({}) - {}",
                    record.id, record.scenario_id, record.tool, record.outcome
                );
            }
        }
        Commands::Show { name } => {
            let record = results_db.load_by_id(name)?;
            match record {
                Some(r) => {
                    println!("Run ID: {}", r.id);
                    println!("Scenario: {}", r.scenario_id);
                    println!("Tool: {}", r.tool);
                    println!("Timestamp: {}", r.timestamp);
                    println!("Duration: {:.2}s", r.duration_secs);
                    println!("Cost: ${:.4}", r.cost_usd);
                    println!("Outcome: {}", r.outcome);
                    println!(
                        "Gates: {}/{}",
                        r.metrics.gates_passed, r.metrics.gates_total
                    );
                    println!("Notes: {}", r.metrics.note_count);
                    println!("Links: {}", r.metrics.link_count);
                    if let Some(score) = r.judge_score {
                        println!("Judge Score: {:.2}", score);
                    }
                    println!("Transcript: {}", r.transcript_path);
                }
                None => println!("Run not found: {}", name),
            }
        }
        Commands::Compare { run_ids } => {
            if run_ids.len() != 2 {
                anyhow::bail!("Compare requires exactly 2 run IDs");
            }

            let r1 = results_db.load_by_id(&run_ids[0])?;
            let r2 = results_db.load_by_id(&run_ids[1])?;

            match (r1, r2) {
                (Some(run1), Some(run2)) => {
                    let report = results::compare_runs(&run1, &run2);
                    print_regression_report(&report);
                }
                _ => anyhow::bail!("One or both runs not found"),
            }
        }
        Commands::Clean => {
            println!("Cleaning cache...");
            cache.clear()?;
            println!("Cache cleared");
        }
    }
    Ok(())
}

fn print_result_summary(record: &ResultRecord) {
    println!("\n--- Result Summary ---");
    println!("ID: {}", record.id);
    println!("Scenario: {}", record.scenario_id);
    println!("Tool: {}", record.tool);
    println!("Outcome: {}", record.outcome);
    println!(
        "Gates: {}/{}",
        record.metrics.gates_passed, record.metrics.gates_total
    );
    println!("Notes: {}", record.metrics.note_count);
    println!("Links: {}", record.metrics.link_count);
    println!("Duration: {:.2}s", record.duration_secs);
    println!(
        "Commands: {} ({} unique, {} errors, {} help, {} retries)",
        record.metrics.efficiency.total_commands,
        record.metrics.efficiency.unique_commands,
        record.metrics.efficiency.error_count,
        record.metrics.efficiency.help_invocations,
        record.metrics.efficiency.retry_count
    );
    println!(
        "First-try success: {:.0}%, iteration ratio: {:.2}",
        record.metrics.efficiency.first_try_success_rate * 100.0,
        record.metrics.efficiency.iteration_ratio
    );
    if let Some(score) = record.judge_score {
        println!("Judge Score: {:.2}", score);
    }
}

fn print_regression_report(report: &RegressionReport) {
    println!("\n--- Regression Report ---");
    println!("Current: {}", report.run_id);
    println!("Baseline: {}", report.baseline_id);

    if let Some(score_change) = report.score_change_pct {
        println!("Score change: {:.1}%", score_change);
    }

    println!("Cost change: {:.1}%", report.cost_change_pct);

    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }

    if !report.alerts.is_empty() {
        println!("\nAlerts:");
        for alert in &report.alerts {
            println!("  - {}", alert);
        }
    }
}
