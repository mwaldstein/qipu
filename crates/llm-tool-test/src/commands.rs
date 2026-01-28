use crate::evaluation::ScoreTier;
use crate::output;
use crate::results::{Cache, ResultsDB};
use crate::run;
use crate::scenario::load;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn find_scenarios(dir: &Path, scenarios: &mut Vec<(String, PathBuf)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yaml" {
                        if let Ok(s) = load(&path) {
                            scenarios.push((s.name.clone(), path));
                        }
                    }
                }
            } else if path.is_dir() {
                find_scenarios(&path, scenarios);
            }
        }
    }
}

pub fn handle_run_command(
    scenario: &Option<String>,
    all: bool,
    tags: &[String],
    tier: &usize,
    tool: &str,
    model: &Option<String>,
    tools: &Option<String>,
    models: &Option<String>,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    judge_model: &Option<String>,
    base_dir: &PathBuf,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<()> {
    if std::env::var("LLM_TOOL_TEST_ENABLED").is_err() {
        anyhow::bail!(
            "LLM testing is disabled for safety. Set LLM_TOOL_TEST_ENABLED=1 to run scenarios."
        );
    }

    if let Some(model) = judge_model {
        std::env::set_var("LLM_TOOL_TEST_JUDGE", model);
    }

    let scenarios_to_run = if all {
        let mut scenarios = Vec::new();
        let fixtures_dir = if PathBuf::from("crates/llm-tool-test/fixtures").exists() {
            PathBuf::from("crates/llm-tool-test/fixtures")
        } else {
            PathBuf::from("fixtures")
        };
        if fixtures_dir.exists() {
            find_scenarios(&fixtures_dir, &mut scenarios);
        }

        let mut filtered_scenarios = Vec::new();
        for (name, path) in scenarios {
            let s = load(&path)?;

            let tags_match = if tags.is_empty() {
                true
            } else {
                tags.iter().all(|tag| s.tags.contains(tag))
            };

            let tier_match = &s.tier <= tier;

            if tags_match && tier_match {
                filtered_scenarios.push((name, path));
            }
        }
        filtered_scenarios
    } else if let Some(path) = scenario {
        let s = load(path)?;
        vec![(s.name.clone(), PathBuf::from(path))]
    } else {
        println!("No scenario specified. Use --scenario <path> or --all");
        return Ok(());
    };

    for (name, path) in scenarios_to_run {
        let s = load(&path)?;
        println!("Loaded scenario: {}", name);

        let matrix = crate::build_tool_matrix(tools, models, tool, model, &s.tool_matrix);

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
    }

    Ok(())
}

pub fn handle_list_command(
    tags: &[String],
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

    fn find_scenarios(
        dir: &std::path::Path,
        scenarios: &mut Vec<(PathBuf, String, usize, String, Vec<String>)>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" {
                            if let Ok(s) = load(&path) {
                                scenarios.push((
                                    path,
                                    s.name.clone(),
                                    s.tier,
                                    s.description,
                                    s.tags,
                                ));
                            }
                        }
                    }
                } else if path.is_dir() {
                    find_scenarios(&path, scenarios);
                }
            }
        }
    }

    let fixtures_dir = if PathBuf::from("crates/llm-tool-test/fixtures").exists() {
        PathBuf::from("crates/llm-tool-test/fixtures")
    } else {
        PathBuf::from("fixtures")
    };
    if fixtures_dir.exists() {
        find_scenarios(&fixtures_dir, &mut scenarios);
    }

    let filtered_scenarios: Vec<_> = scenarios
        .iter()
        .filter(|(_, _, scenario_tier, _, scenario_tags)| {
            let tier_match = scenario_tier <= tier;
            let tags_match = if tags.is_empty() {
                true
            } else {
                tags.iter().all(|tag| scenario_tags.contains(tag))
            };
            tier_match && tags_match
        })
        .collect();

    let tier_label = match *tier {
        0 => "smoke",
        1 => "quick",
        2 => "standard",
        3 => "comprehensive",
        _ => "unknown",
    };
    println!("Available scenarios (tier {}):", tier_label);
    for (_path, name, _scenario_tier, description, scenario_tags) in &filtered_scenarios {
        let tags_str = if scenario_tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", scenario_tags.join(", "))
        };
        println!("  [{}] {}{} - {}", tier_label, name, tags_str, description);
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

pub fn handle_report_command(results_db: &ResultsDB) -> anyhow::Result<()> {
    let records = results_db.load_all()?;

    if records.is_empty() {
        println!("No test runs found");
        return Ok(());
    }

    // Group by scenario
    let mut by_scenario: HashMap<String, Vec<_>> = HashMap::new();
    for record in &records {
        by_scenario
            .entry(record.scenario_id.clone())
            .or_default()
            .push(record);
    }

    println!("# LLM Tool Test Summary Report\n");
    println!("Generated: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    println!("Total runs: {}\n", records.len());

    // Overall statistics
    let total_cost: f64 = records.iter().map(|r| r.cost_usd).sum();
    let avg_cost = total_cost / records.len() as f64;
    let total_duration: f64 = records.iter().map(|r| r.duration_secs).sum();
    let avg_duration = total_duration / records.len() as f64;
    let pass_count = records.iter().filter(|r| r.gates_passed).count();
    let pass_rate = (pass_count as f64 / records.len() as f64) * 100.0;

    println!("## Overall Statistics");
    println!(
        "- Pass rate: {:.1}% ({}/{})",
        pass_rate,
        pass_count,
        records.len()
    );
    println!("- Total cost: ${:.2}", total_cost);
    println!("- Average cost per run: ${:.4}", avg_cost);
    println!("- Total duration: {:.1}s", total_duration);
    println!("- Average duration per run: {:.1}s\n", avg_duration);

    // Per-scenario breakdown
    println!("## Scenarios\n");
    let mut scenario_names: Vec<_> = by_scenario.keys().collect();
    scenario_names.sort();

    for scenario_name in scenario_names {
        let scenario_records = &by_scenario[scenario_name];
        let scenario_pass_count = scenario_records.iter().filter(|r| r.gates_passed).count();
        let scenario_pass_rate =
            (scenario_pass_count as f64 / scenario_records.len() as f64) * 100.0;
        let scenario_avg_score: f64 = scenario_records
            .iter()
            .filter_map(|r| r.judge_score)
            .sum::<f64>()
            / scenario_records
                .iter()
                .filter(|r| r.judge_score.is_some())
                .count()
                .max(1) as f64;

        println!("### {}", scenario_name);
        println!("- Runs: {}", scenario_records.len());
        println!(
            "- Pass rate: {:.1}% ({}/{})",
            scenario_pass_rate,
            scenario_pass_count,
            scenario_records.len()
        );
        if scenario_records.iter().any(|r| r.judge_score.is_some()) {
            println!("- Average judge score: {:.2}", scenario_avg_score);
        }

        // Group by tool
        let mut by_tool: HashMap<String, Vec<_>> = HashMap::new();
        for record in scenario_records.iter() {
            by_tool
                .entry(record.tool.clone())
                .or_default()
                .push(*record);
        }

        for (tool, tool_records) in by_tool.iter() {
            let tool_pass_count = tool_records.iter().filter(|r| r.gates_passed).count();
            let tool_pass_rate = (tool_pass_count as f64 / tool_records.len() as f64) * 100.0;
            let latest = tool_records.iter().max_by_key(|r| r.timestamp).unwrap();

            println!(
                "  - {}: {:.1}% pass rate, latest: {} ({})",
                tool, tool_pass_rate, latest.outcome, latest.id
            );
        }
        println!();
    }

    // Recent runs
    let mut sorted_records = records.clone();
    sorted_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    println!("## Recent Runs (last 10)\n");
    for record in sorted_records.iter().take(10) {
        let status = if record.gates_passed { "✓" } else { "✗" };
        println!(
            "- {} [{}] {} / {} - {} (${:.4}, {:.1}s)",
            status,
            record.id,
            record.scenario_id,
            record.tool,
            record.outcome,
            record.cost_usd,
            record.duration_secs
        );
    }

    Ok(())
}

pub fn handle_clean_command(
    cache: &Cache,
    older_than: &Option<String>,
    base_dir: &PathBuf,
) -> anyhow::Result<()> {
    let cutoff_time = if let Some(duration_str) = older_than {
        let duration = parse_duration(duration_str)?;
        Some(Utc::now() - duration)
    } else {
        None
    };

    // Clean cache
    println!("Cleaning cache...");
    cache.clear()?;
    println!("Cache cleared");

    // Clean old transcripts
    let transcripts_dir = base_dir.join("transcripts");
    if !transcripts_dir.exists() {
        println!("No transcripts directory found");
        return Ok(());
    }

    let mut removed_count = 0;
    let mut kept_count = 0;

    for entry in std::fs::read_dir(&transcripts_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Check if we should delete based on age
        let should_delete = if let Some(cutoff) = cutoff_time {
            // Get the modification time of the directory
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_datetime = chrono::DateTime::<Utc>::from(modified);
                    modified_datetime < cutoff
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // If no cutoff time specified, delete all
            true
        };

        if should_delete {
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!("Warning: Failed to remove {}: {}", path.display(), e);
            } else {
                removed_count += 1;
            }
        } else {
            kept_count += 1;
        }
    }

    if let Some(duration_str) = older_than {
        println!(
            "Cleaned {} transcript(s) older than {}, kept {}",
            removed_count, duration_str, kept_count
        );
    } else {
        println!("Cleaned {} transcript(s)", removed_count);
    }

    Ok(())
}

fn parse_duration(s: &str) -> anyhow::Result<Duration> {
    let re = regex::Regex::new(r"^(\d+)([dhm])$")?;
    let caps = re.captures(s).ok_or_else(|| {
        anyhow::anyhow!("Invalid duration format. Use format like '30d', '7d', '1h'")
    })?;

    let value: i64 = caps[1].parse()?;
    let unit = &caps[2];

    let duration = match unit {
        "d" => Duration::days(value),
        "h" => Duration::hours(value),
        "m" => Duration::minutes(value),
        _ => anyhow::bail!("Invalid duration unit. Use 'd' (days), 'h' (hours), or 'm' (minutes)"),
    };

    Ok(duration)
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

pub fn handle_baseline_set_command(run_id: &str, results_db: &ResultsDB) -> anyhow::Result<()> {
    let record = results_db.load_by_id(run_id)?;
    match record {
        Some(r) => {
            results_db.set_baseline(&r.scenario_id, &r.tool, run_id)?;
            println!(
                "Baseline set: {} for scenario '{}' with tool '{}'",
                run_id, r.scenario_id, r.tool
            );
        }
        None => anyhow::bail!("Run not found: {}", run_id),
    }
    Ok(())
}

pub fn handle_baseline_clear_command(
    scenario_id: &str,
    tool: &str,
    results_db: &ResultsDB,
) -> anyhow::Result<()> {
    results_db.clear_baseline(scenario_id, tool)?;
    println!(
        "Baseline cleared for scenario '{}' with tool '{}'",
        scenario_id, tool
    );
    Ok(())
}

pub fn handle_baseline_list_command(results_db: &ResultsDB) -> anyhow::Result<()> {
    let baselines = results_db.list_baselines()?;
    if baselines.is_empty() {
        println!("No baselines configured");
    } else {
        println!("Configured baselines ({}):", baselines.len());
        for (key, run_id) in baselines {
            println!("  {} -> {}", key, run_id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::{
        EfficiencyMetricsRecord, EvaluationMetricsRecord, QualityMetricsRecord, ResultRecord,
    };
    use tempfile::TempDir;

    #[test]
    fn test_find_scenarios() {
        let temp_dir = std::path::PathBuf::from("/tmp/test_scenarios_find");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("subdir");
        std::fs::create_dir_all(&sub_dir).unwrap();

        let yaml1 = temp_dir.join("scenario1.yaml");
        let yaml2 = sub_dir.join("scenario2.yaml");
        let txt = temp_dir.join("not_a_scenario.txt");

        let scenario1_content = r#"
name: test1
description: "Test scenario 1"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario2_content = r#"
name: test2
description: "Test scenario 2"
template_folder: qipu
task:
  prompt: "Test"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;

        std::fs::write(&yaml1, scenario1_content).unwrap();
        std::fs::write(&yaml2, scenario2_content).unwrap();
        std::fs::write(&txt, "not a scenario").unwrap();

        let mut scenarios = Vec::new();
        find_scenarios(&temp_dir, &mut scenarios);

        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.iter().any(|(name, _)| name == "test1"));
        assert!(scenarios.iter().any(|(name, _)| name == "test2"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    fn create_test_result_record(id: &str) -> ResultRecord {
        ResultRecord {
            id: id.to_string(),
            scenario_id: "test-scenario".to_string(),
            scenario_hash: "hash123".to_string(),
            tool: "opencode".to_string(),
            model: "gpt-4o".to_string(),
            qipu_commit: "abc123".to_string(),
            timestamp: Utc::now(),
            duration_secs: 45.5,
            cost_usd: 0.01,
            gates_passed: true,
            metrics: EvaluationMetricsRecord {
                gates_passed: 2,
                gates_total: 2,
                note_count: 1,
                link_count: 0,
                details: vec![],
                efficiency: EfficiencyMetricsRecord {
                    total_commands: 3,
                    unique_commands: 2,
                    error_count: 0,
                    retry_count: 1,
                    help_invocations: 0,
                    first_try_success_rate: 1.0,
                    iteration_ratio: 1.5,
                },
                quality: QualityMetricsRecord {
                    avg_title_length: 10.0,
                    avg_body_length: 50.0,
                    avg_tags_per_note: 2.0,
                    notes_without_tags: 0,
                    links_per_note: 0.0,
                    orphan_notes: 1,
                    link_type_diversity: 0,
                    type_distribution: HashMap::new(),
                    total_notes: 1,
                    total_links: 0,
                },
                composite_score: 0.9,
            },
            judge_score: Some(0.9),
            outcome: "PASS".to_string(),
            transcript_path: "/path/to/transcript.txt".to_string(),
            cache_key: Some("cache-key-123".to_string()),
            human_review: None,
        }
    }

    #[test]
    fn test_handle_review_command_valid_run() {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());

        let record = create_test_result_record("run-1");
        db.append(&record).unwrap();

        let dimensions = vec![("accuracy".to_string(), 0.9), ("clarity".to_string(), 0.8)];
        let notes = Some("Good work".to_string());

        let result = handle_review_command("run-1", &dimensions, &notes, &db);
        assert!(result.is_ok());

        let loaded = db.load_by_id("run-1").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert!(loaded.human_review.is_some());
        let review = loaded.human_review.unwrap();
        assert_eq!(review.dimensions.get("accuracy"), Some(&0.9));
        assert_eq!(review.dimensions.get("clarity"), Some(&0.8));
        assert_eq!(review.notes, Some("Good work".to_string()));
    }

    #[test]
    fn test_handle_review_command_nonexistent_run() {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());

        let dimensions = vec![("accuracy".to_string(), 0.9)];
        let notes = None;

        let result = handle_review_command("nonexistent", &dimensions, &notes, &db);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Run not found"));
    }

    #[test]
    fn test_handle_list_command_pending_review() {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());

        let record1 = create_test_result_record("run-1");
        let record2 = create_test_result_record("run-2");
        let record3 = create_test_result_record("run-3");

        db.append(&record1).unwrap();
        db.append(&record2).unwrap();
        db.append(&record3).unwrap();

        let result = handle_list_command(&[], &0, true, &db);
        assert!(result.is_ok());

        let loaded = db.load_pending_review().unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn test_handle_list_command_pending_review_empty() {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());

        let result = handle_list_command(&[], &0, true, &db);
        assert!(result.is_ok());

        let loaded = db.load_pending_review().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_handle_review_command_updates_pending_status() {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());

        let record1 = create_test_result_record("run-1");
        let record2 = create_test_result_record("run-2");

        db.append(&record1).unwrap();
        db.append(&record2).unwrap();

        let pending = db.load_pending_review().unwrap();
        assert_eq!(pending.len(), 2);

        let dimensions = vec![("accuracy".to_string(), 0.9)];
        handle_review_command("run-1", &dimensions, &None, &db).unwrap();

        let pending = db.load_pending_review().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "run-2");
    }
}
