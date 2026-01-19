mod adapter;
mod cli;
mod evaluation;
mod fixture;
mod judge;
mod results;
mod scenario;
mod session;
mod store_analysis;
mod transcript;

use adapter::{
    amp::AmpAdapter, claude_code::ClaudeCodeAdapter, opencode::OpenCodeAdapter, ToolAdapter,
};
use chrono::Utc;
use clap::Parser;
use cli::{Cli, Commands};
use results::{
    Cache, CacheKey, EfficiencyMetricsRecord, EvaluationMetricsRecord, GateResultRecord,
    QualityMetricsRecord, RegressionReport, ResultRecord, ResultsDB,
};
use scenario::ToolConfig;
use std::collections::HashMap;
use std::iter::Iterator;
use std::time::Instant;

#[derive(Debug, Clone)]
struct ToolModelConfig {
    tool: String,
    model: String,
}

fn build_tool_matrix(
    cli_tools: &Option<String>,
    cli_models: &Option<String>,
    cli_tool: &str,
    cli_model: &Option<String>,
    scenario_matrix: &Option<Vec<ToolConfig>>,
) -> Vec<ToolModelConfig> {
    if let (Some(tools_str), Some(models_str)) = (cli_tools, cli_models) {
        let tools: Vec<String> = tools_str.split(',').map(|s| s.trim().to_string()).collect();
        let models: Vec<String> = models_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut matrix = Vec::new();
        for tool in &tools {
            for model in &models {
                matrix.push(ToolModelConfig {
                    tool: tool.clone(),
                    model: model.clone(),
                });
            }
        }
        matrix
    } else if let Some(scenario_matrix) = scenario_matrix {
        let mut matrix = Vec::new();
        for config in scenario_matrix {
            let models = if config.models.is_empty() {
                vec!["default".to_string()]
            } else {
                config.models.clone()
            };
            for model in models {
                matrix.push(ToolModelConfig {
                    tool: config.tool.clone(),
                    model,
                });
            }
        }
        matrix
    } else {
        vec![ToolModelConfig {
            tool: cli_tool.to_string(),
            model: cli_model.as_deref().unwrap_or("default").to_string(),
        }]
    }
}

fn run_single_scenario(
    s: &scenario::Scenario,
    tool: &str,
    model: &str,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    _base_dir: &std::path::Path,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<ResultRecord> {
    let scenario_path = format!("fixtures/{}/{}.yaml", s.fixture, s.name);
    let scenario_yaml = std::fs::read_to_string(&scenario_path)?;
    let prompt = s.task.prompt.clone();
    let qipu_version = results::get_qipu_version()?;
    let cache_key = CacheKey::compute(&scenario_yaml, &prompt, tool, model, &qipu_version);

    if !no_cache {
        if let Some(cached) = cache.get(&cache_key) {
            println!("Cache HIT! Using cached result: {}", cached.id);
            print_result_summary(&cached);
            return Ok(cached);
        }
    }

    let adapter: Box<dyn ToolAdapter> = match tool {
        "amp" => Box::new(AmpAdapter),
        "claude-code" => Box::new(ClaudeCodeAdapter),
        "opencode" => Box::new(OpenCodeAdapter),
        _ => anyhow::bail!("Unknown tool: {}", tool),
    };

    if !dry_run {
        println!("Checking availability for tool: {}", tool);
        if let Err(e) = adapter.check_availability() {
            anyhow::bail!("Tool unavailable: {}", e);
        }

        println!("Setting up environment for fixture: {}", s.fixture);
        let env = fixture::TestEnv::new(&s.name)?;
        env.setup_fixture(&s.fixture)?;
        println!("Environment created at: {:?}", env.root);

        if let Some(setup_steps) = &s.setup {
            println!("Running {} setup step(s)...", setup_steps.len());
            let runner = session::SessionRunner::new();
            for (i, step) in setup_steps.iter().enumerate() {
                let args: Vec<&str> = step.args.iter().map(|s| s.as_str()).collect();
                println!(
                    "  Step {}/{}: {} {}",
                    i + 1,
                    setup_steps.len(),
                    step.command,
                    args.join(" ")
                );
                let (output, exit_code) =
                    runner.run_command(&step.command, &args, &env.root, timeout_secs)?;
                if exit_code != 0 {
                    anyhow::bail!(
                        "Setup step {}/{} failed: {} exited with code {}. Output: {}",
                        i + 1,
                        setup_steps.len(),
                        step.command,
                        exit_code,
                        output
                    );
                }
            }
            println!("Setup complete.");
        }

        let start_time = Instant::now();
        println!("Running tool '{}' with model '{}'...", tool, model);
        let (output, exit_code, cost) = adapter.run(s, &env.root, Some(model), timeout_secs)?;
        let duration = start_time.elapsed();

        let transcript_dir = env.root.join("artifacts");
        std::fs::create_dir_all(&transcript_dir)?;
        let writer = transcript::TranscriptWriter::new(transcript_dir.clone())?;
        writer.write_raw(&output)?;
        writer.append_event(&serde_json::json!({
            "type": "execution",
            "tool": tool,
            "output": output,
            "exit_code": exit_code,
            "cost_usd": cost
        }))?;

        println!("Running evaluation...");
        let metrics = evaluation::evaluate(s, &env.root)?;
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
            tool: tool.to_string(),
            model: model.to_string(),
            qipu_commit: qipu_version.clone(),
            timestamp: Utc::now(),
            duration_secs: duration.as_secs_f64(),
            cost_usd: cost,
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
                quality: QualityMetricsRecord {
                    avg_title_length: metrics.quality.avg_title_length,
                    avg_body_length: metrics.quality.avg_body_length,
                    avg_tags_per_note: metrics.quality.avg_tags_per_note,
                    notes_without_tags: metrics.quality.notes_without_tags,
                    links_per_note: metrics.quality.links_per_note,
                    orphan_notes: metrics.quality.orphan_notes,
                    link_type_diversity: metrics.quality.link_type_diversity,
                    type_distribution: metrics.quality.type_distribution,
                    total_notes: metrics.quality.total_notes,
                    total_links: metrics.quality.total_links,
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

        Ok(record)
    } else {
        println!("Dry run - skipping execution");
        anyhow::bail!("Dry run not supported in matrix mode");
    }
}

fn print_matrix_summary(results: &[(ToolModelConfig, anyhow::Result<ResultRecord>)]) {
    println!("\n--- Matrix Summary ---");

    let mut table: HashMap<String, HashMap<String, String>> = HashMap::new();

    for (config, result) in results {
        let outcome = match result {
            Ok(record) => record.outcome.clone(),
            Err(e) => format!("Error: {}", e),
        };

        table
            .entry(config.tool.clone())
            .or_default()
            .insert(config.model.clone(), outcome);
    }

    println!("{:<20} |", "Tool");
    let all_models: std::collections::BTreeSet<_> = table
        .values()
        .flat_map(|models| models.keys().cloned())
        .collect();

    for model in &all_models {
        print!(" {:<20} |", model);
    }
    println!();

    println!("{}", "-".repeat(22));
    for _ in &all_models {
        println!("{}", "-".repeat(22));
    }
    println!();

    let mut tools: Vec<_> = table.keys().collect();
    tools.sort();

    for tool in tools {
        print!("{:<20} |", tool);
        for model in &all_models {
            let default = "-";
            let outcome = table
                .get(tool)
                .and_then(|m| m.get(model))
                .map_or(default, |v| v.as_str());
            print!(" {:<20} |", outcome);
        }
        println!();
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dir = std::path::PathBuf::from("target/llm_test_runs");
    let results_db = ResultsDB::new(&base_dir);
    let cache = Cache::new(&base_dir);

    match &cli.command {
        Commands::Run {
            scenario,
            tags: _,
            tier,
            tool,
            model,
            tools,
            models,
            max_usd: _,
            dry_run,
            no_cache,
            judge_model,
            timeout_secs,
        } => {
            if let Some(model) = judge_model {
                std::env::set_var("LLM_TOOL_TEST_JUDGE", model);
            }

            if let Some(path) = scenario {
                let s = scenario::load(path)?;
                println!("Loaded scenario: {}", s.name);

                let matrix = build_tool_matrix(tools, models, tool, model, &s.tool_matrix);

                if matrix.len() > 1 {
                    println!("Matrix run: {} tool×model combinations", matrix.len());
                }

                let mut results = Vec::new();

                for config in &matrix {
                    println!("\n=== Running: {} / {} ===", config.tool, config.model);

                    let result = run_single_scenario(
                        &s,
                        &config.tool,
                        &config.model,
                        *dry_run,
                        *no_cache,
                        *timeout_secs,
                        &base_dir,
                        &results_db,
                        &cache,
                    );

                    results.push((config.clone(), result));
                }

                if matrix.len() > 1 {
                    print_matrix_summary(&results);
                }
            } else {
                println!("No scenario specified. Use --scenario <path>");
            }
        }
        Commands::List { tags: _, tier } => {
            let mut scenarios = Vec::new();

            fn find_scenarios(dir: &std::path::Path, scenarios: &mut Vec<(String, usize, String)>) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "yaml" {
                                    if let Ok(s) = scenario::load(&path) {
                                        scenarios.push((s.name.clone(), s.tier, s.description));
                                    }
                                }
                            }
                        } else if path.is_dir() {
                            find_scenarios(&path, scenarios);
                        }
                    }
                }
            }

            let fixtures_dir = std::path::PathBuf::from("fixtures");
            if fixtures_dir.exists() {
                find_scenarios(&fixtures_dir, &mut scenarios);
            }

            scenarios.sort_by(|a, b| a.1.cmp(&b.1));

            let tier_label = match *tier {
                0 => "smoke",
                1 => "quick",
                2 => "standard",
                3 => "comprehensive",
                _ => "unknown",
            };
            println!("Available scenarios (tier {}):", tier_label);
            for (name, _tier, description) in &scenarios {
                println!("  [{}] {} - {}", tier_label, name, description);
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
