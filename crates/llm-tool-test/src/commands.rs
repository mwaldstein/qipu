use crate::evaluation::ScoreTier;
use crate::output;
use crate::results::{Cache, ResultsDB};
use crate::run;
use crate::scenario::{load, ToolConfig};
use chrono::Utc;
use std::path::PathBuf;

pub fn handle_run_command(
    scenario: &Option<String>,
    tool: &str,
    model: &Option<String>,
    tools: &Option<String>,
    models: &Option<String>,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    judge_model: &Option<String>,
    tool_matrix: &Option<Vec<ToolConfig>>,
    base_dir: &PathBuf,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<()> {
    if let Some(model) = judge_model {
        std::env::set_var("LLM_TOOL_TEST_JUDGE", model);
    }

    if let Some(path) = scenario {
        let s = load(path)?;
        println!("Loaded scenario: {}", s.name);

        let matrix = crate::build_tool_matrix(tools, models, tool, model, tool_matrix);

        if matrix.len() > 1 {
            println!("Matrix run: {} tool×model combinations", matrix.len());
        }

        let mut results = Vec::new();

        for config in &matrix {
            println!("\n=== Running: {} / {} ===", config.tool, config.model);

            let result = run::run_single_scenario(
                &s,
                &config.tool,
                &config.model,
                dry_run,
                no_cache,
                timeout_secs,
                base_dir,
                results_db,
                cache,
            );

            results.push((config.clone(), result));
        }

        if matrix.len() > 1 {
            output::print_matrix_summary(&results);
        }
    } else {
        println!("No scenario specified. Use --scenario <path>");
    }

    Ok(())
}

pub fn handle_list_command(
    tier: &usize,
    pending_review: bool,
    results_db: &ResultsDB,
) -> anyhow::Result<()> {
    if pending_review {
        let pending = results_db.load_pending_review()?;
        if pending.is_empty() {
            println!("No runs pending review");
        } else {
            println!("Runs pending review ({}):", pending.len());
            for r in pending {
                println!("  [{}] {} - {} ({})", r.id, r.scenario_id, r.tool, r.model);
            }
        }
        return Ok(());
    }

    let mut scenarios = Vec::new();

    fn find_scenarios(dir: &std::path::Path, scenarios: &mut Vec<(String, usize, String)>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" {
                            if let Ok(s) = load(&path) {
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

    Ok(())
}

pub fn handle_show_command(name: &str, results_db: &ResultsDB) -> anyhow::Result<()> {
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
                let tier = ScoreTier::from_score(score);
                println!("Judge Score: {:.2} ({})", score, tier);
            }
            let composite_tier = ScoreTier::from_score(r.metrics.composite_score);
            println!(
                "Composite Score: {:.2} ({})",
                r.metrics.composite_score, composite_tier
            );
            println!("Transcript: {}", r.transcript_path);
            if let Some(review) = r.human_review {
                println!("Human Review:");
                for (dim, score) in &review.dimensions {
                    println!("  {}: {:.2}", dim, score);
                }
                if let Some(notes) = &review.notes {
                    println!("  Notes: {}", notes);
                }
                println!("  Reviewed: {}", review.timestamp);
            }
        }
        None => println!("Run not found: {}", name),
    }

    Ok(())
}

pub fn handle_compare_command(run_ids: &[String], results_db: &ResultsDB) -> anyhow::Result<()> {
    if run_ids.len() != 2 {
        anyhow::bail!("Compare requires exactly 2 run IDs");
    }

    let r1 = results_db.load_by_id(&run_ids[0])?;
    let r2 = results_db.load_by_id(&run_ids[1])?;

    match (r1, r2) {
        (Some(run1), Some(run2)) => {
            let report = crate::results::compare_runs(&run1, &run2);
            output::print_regression_report(&report);
        }
        _ => anyhow::bail!("One or both runs not found"),
    }

    Ok(())
}

pub fn handle_clean_command(cache: &Cache) -> anyhow::Result<()> {
    println!("Cleaning cache...");
    cache.clear()?;
    println!("Cache cleared");
    Ok(())
}

pub fn handle_review_command(
    run_id: &str,
    dimension: &[(String, f64)],
    notes: &Option<String>,
    results_db: &ResultsDB,
) -> anyhow::Result<()> {
    let record = results_db.load_by_id(&run_id)?;
    match record {
        Some(_) => {
            let dimensions_map: std::collections::HashMap<String, f64> =
                dimension.iter().map(|(k, v)| (k.clone(), *v)).collect();
            let human_review = crate::results::HumanReviewRecord {
                dimensions: dimensions_map,
                notes: notes.clone(),
                timestamp: Utc::now(),
            };

            results_db.update_human_review(&run_id, human_review)?;
            println!("Review added for run: {}", run_id);
        }
        None => anyhow::bail!("Run not found: {}", run_id),
    }

    Ok(())
}
