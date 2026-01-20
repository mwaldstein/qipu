use crate::evaluation::ScoreTier;
use crate::results::{RegressionReport, ResultRecord};
use std::collections::HashMap;

pub fn print_matrix_summary(
    results: &[(
        crate::commands::ToolModelConfig,
        anyhow::Result<ResultRecord>,
    )],
) {
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

pub fn print_result_summary(record: &ResultRecord) {
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
        let tier = ScoreTier::from_score(score);
        println!("Judge Score: {:.2} ({})", score, tier);
    }
    let composite_tier = ScoreTier::from_score(record.metrics.composite_score);
    println!(
        "Composite Score: {:.2} ({})",
        record.metrics.composite_score, composite_tier
    );
    if record.human_review.is_some() {
        println!("Human Review: Yes");
    }
}

pub fn print_regression_report(report: &RegressionReport) {
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
