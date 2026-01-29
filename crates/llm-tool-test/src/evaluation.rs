use crate::judge::{load_rubric, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::store_analysis::QualityMetrics;
use crate::transcript::EfficiencyMetrics;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

pub trait GateEvaluator {
    fn evaluate(&self, env_root: &Path) -> GateResult;
}

impl GateEvaluator for Gate {
    fn evaluate(&self, env_root: &Path) -> GateResult {
        match self {
            Gate::MinNotes { count } => eval_min_notes(*count, env_root),
            Gate::MinLinks { count } => eval_min_links(*count, env_root),
            Gate::SearchHit { query } => eval_search_hit(query, env_root),
            Gate::NoteExists { id } => eval_note_exists(id, env_root),
            Gate::LinkExists {
                from,
                to,
                link_type,
            } => eval_link_exists(from, to, link_type, env_root),
            Gate::TagExists { tag } => eval_tag_exists(tag, env_root),
            Gate::ContentContains { id, substring } => {
                eval_content_contains(id, substring, env_root)
            }
            Gate::CommandSucceeds { command } => eval_command_succeeds(command, env_root),
            Gate::DoctorPasses => eval_doctor_passes(env_root),
            Gate::NoTranscriptErrors => eval_no_transcript_errors(env_root),
        }
    }
}

fn eval_min_notes(count: usize, env_root: &Path) -> GateResult {
    match crate::eval_helpers::count_notes(env_root).context("Failed to count notes") {
        Ok(note_count) => GateResult {
            gate_type: "MinNotes".to_string(),
            passed: note_count >= count,
            message: format!("Expected >= {}, found {}", count, note_count),
        },
        Err(e) => GateResult {
            gate_type: "MinNotes".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_min_links(count: usize, env_root: &Path) -> GateResult {
    match crate::eval_helpers::count_links(env_root).context("Failed to count links") {
        Ok(link_count) => GateResult {
            gate_type: "MinLinks".to_string(),
            passed: link_count >= count,
            message: format!("Expected >= {}, found {}", count, link_count),
        },
        Err(e) => GateResult {
            gate_type: "MinLinks".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_search_hit(query: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::search_hit(query, env_root) {
        Ok(hit) => GateResult {
            gate_type: "SearchHit".to_string(),
            passed: hit,
            message: format!("Query '{}' found: {}", query, hit),
        },
        Err(e) => GateResult {
            gate_type: "SearchHit".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_note_exists(id: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::note_exists(id, env_root) {
        Ok(exists) => GateResult {
            gate_type: "NoteExists".to_string(),
            passed: exists,
            message: format!("Note '{}' exists: {}", id, exists),
        },
        Err(e) => GateResult {
            gate_type: "NoteExists".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_link_exists(from: &str, to: &str, link_type: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::link_exists(from, to, link_type, env_root) {
        Ok(exists) => GateResult {
            gate_type: "LinkExists".to_string(),
            passed: exists,
            message: format!(
                "Link {} --[{}]--> {} exists: {}",
                from, link_type, to, exists
            ),
        },
        Err(e) => GateResult {
            gate_type: "LinkExists".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_tag_exists(tag: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::tag_exists(tag, env_root) {
        Ok(exists) => GateResult {
            gate_type: "TagExists".to_string(),
            passed: exists,
            message: format!("Tag '{}' exists: {}", tag, exists),
        },
        Err(e) => GateResult {
            gate_type: "TagExists".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_content_contains(id: &str, substring: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::content_contains(id, substring, env_root) {
        Ok(contains) => GateResult {
            gate_type: "ContentContains".to_string(),
            passed: contains,
            message: format!("Note '{}' contains '{}': {}", id, substring, contains),
        },
        Err(e) => GateResult {
            gate_type: "ContentContains".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_command_succeeds(command: &str, env_root: &Path) -> GateResult {
    match crate::eval_helpers::command_succeeds(command, env_root) {
        Ok(succeeds) => GateResult {
            gate_type: "CommandSucceeds".to_string(),
            passed: succeeds,
            message: format!("Command '{}' succeeded: {}", command, succeeds),
        },
        Err(e) => GateResult {
            gate_type: "CommandSucceeds".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_doctor_passes(env_root: &Path) -> GateResult {
    match crate::eval_helpers::doctor_passes(env_root) {
        Ok(passes) => GateResult {
            gate_type: "DoctorPasses".to_string(),
            passed: passes,
            message: format!("Store passes 'qipu doctor': {}", passes),
        },
        Err(e) => GateResult {
            gate_type: "DoctorPasses".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

fn eval_no_transcript_errors(env_root: &Path) -> GateResult {
    match crate::eval_helpers::no_transcript_errors(env_root) {
        Ok(no_errors) => GateResult {
            gate_type: "NoTranscriptErrors".to_string(),
            passed: no_errors,
            message: format!("Transcript has no command errors: {}", no_errors),
        },
        Err(e) => GateResult {
            gate_type: "NoTranscriptErrors".to_string(),
            passed: false,
            message: format!("Evaluation error: {:#}", e),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTier {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

impl ScoreTier {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ScoreTier::Excellent
        } else if score >= 0.7 {
            ScoreTier::Good
        } else if score >= 0.5 {
            ScoreTier::Acceptable
        } else {
            ScoreTier::Poor
        }
    }
}

impl fmt::Display for ScoreTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoreTier::Excellent => write!(f, "Excellent"),
            ScoreTier::Good => write!(f, "Good"),
            ScoreTier::Acceptable => write!(f, "Acceptable"),
            ScoreTier::Poor => write!(f, "Poor"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub gates_passed: usize,
    pub gates_total: usize,
    pub note_count: usize,
    pub link_count: usize,
    pub details: Vec<GateResult>,
    pub judge_score: Option<f64>,
    pub judge_response: Option<JudgeResponse>,
    pub efficiency: EfficiencyMetrics,
    pub quality: QualityMetrics,
    pub composite_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

pub fn evaluate(scenario: &Scenario, env_root: &Path, no_judge: bool) -> Result<EvaluationMetrics> {
    println!("Evaluating results for scenario: {}", scenario.name);

    let mut details = Vec::new();
    let mut gates_passed = 0;

    for gate in &scenario.evaluation.gates {
        let result = gate.evaluate(env_root);

        if result.passed {
            println!("Gate {} passed: {}", result.gate_type, result.message);
            gates_passed += 1;
        } else {
            println!("Gate {} FAILED: {}", result.gate_type, result.message);
        }
        details.push(result);
    }

    let mut judge_score = None;
    let mut judge_response = None;

    if let Some(judge_config) = &scenario.evaluation.judge {
        if judge_config.enabled && !no_judge {
            println!("Running LLM-as-judge evaluation...");
            let rubric_path = env_root.join(&judge_config.rubric);
            let _rubric = load_rubric(&rubric_path)
                .with_context(|| format!("Failed to load rubric from {}", judge_config.rubric))?;

            let transcript_path = env_root.join("artifacts/transcript.raw.txt");
            let store_path = env_root.join("artifacts/store_snapshot/export.json");

            let runner = crate::session::SessionRunner::new();
            let prompt = format!(
                r#"Evaluate this LLM tool interaction.

Task: {}

Files to review:
- @{} - The interaction transcript
- @{} - Store state after interaction

Use the rubric at {} for evaluation.

Return evaluation as JSON with this structure:
{{
  "scores": {{
    "criterion_id": <score_0_to_1>,
    ...
  }},
  "weighted_score": <weighted_average_0_to_1>,
  "confidence": <confidence_0_to_1>,
  "issues": ["issue1", "issue2", ...],
  "highlights": ["good_practice1", "good_practice2", ...]
}}

Provide JSON only, no additional text."#,
                scenario.task.prompt,
                transcript_path.display(),
                store_path.display(),
                rubric_path.display()
            );

            let (output, exit_code) = runner
                .run_command("opencode", &["run", &prompt], env_root, 300)
                .context("Judge execution failed")?;

            if exit_code != 0 {
                anyhow::bail!("Judge exited with code {}: {}", exit_code, output);
            }

            let response: JudgeResponse = serde_json::from_str(&output)
                .with_context(|| format!("Failed to parse judge response: {}", output))?;

            println!(
                "Judge score: {:.2} (confidence: {:.2})",
                response.weighted_score, response.confidence
            );
            if !response.issues.is_empty() {
                println!("Issues: {}", response.issues.join(", "));
            }
            if !response.highlights.is_empty() {
                println!("Highlights: {}", response.highlights.join(", "));
            }

            judge_score = Some(response.weighted_score);
            judge_response = Some(response);
        }
    }

    let efficiency =
        crate::eval_helpers::compute_efficiency_metrics(env_root).unwrap_or_else(|_| {
            EfficiencyMetrics {
                total_commands: 0,
                unique_commands: 0,
                error_count: 0,
                retry_count: 0,
                help_invocations: 0,
                first_try_success_rate: 0.0,
                iteration_ratio: 0.0,
            }
        });
    let quality =
        crate::eval_helpers::compute_quality_metrics(env_root).unwrap_or_else(|_| QualityMetrics {
            avg_title_length: 0.0,
            avg_body_length: 0.0,
            avg_tags_per_note: 0.0,
            notes_without_tags: 0,
            links_per_note: 0.0,
            orphan_notes: 0,
            link_type_diversity: 0,
            type_distribution: std::collections::HashMap::new(),
            total_notes: 0,
            total_links: 0,
        });
    let composite_score = crate::eval_helpers::compute_composite_score(
        judge_score,
        gates_passed,
        scenario.evaluation.gates.len(),
        &efficiency,
        &quality,
    );

    let note_count = crate::eval_helpers::count_notes(env_root).unwrap_or(0);
    let link_count = crate::eval_helpers::count_links(env_root).unwrap_or(0);

    let metrics = EvaluationMetrics {
        gates_passed,
        gates_total: scenario.evaluation.gates.len(),
        note_count,
        link_count,
        details,
        judge_score,
        judge_response,
        efficiency,
        quality,
        composite_score,
    };

    Ok(metrics)
}
