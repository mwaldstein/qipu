use crate::judge::{load_rubric, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::store_analysis::{QualityMetrics, StoreAnalyzer};
use crate::transcript::EfficiencyMetrics;
use crate::transcript::TranscriptAnalyzer;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::process::Command;

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
    match count_notes(env_root).context("Failed to count notes") {
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
    match count_links(env_root).context("Failed to count links") {
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
    match search_hit(query, env_root) {
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
    match note_exists(id, env_root) {
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
    match link_exists(from, to, link_type, env_root) {
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
    match tag_exists(tag, env_root) {
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
    match content_contains(id, substring, env_root) {
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
    match command_succeeds(command, env_root) {
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
    match doctor_passes(env_root) {
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
    match no_transcript_errors(env_root) {
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

    // Compute supplementary metrics (fallback to defaults on error)
    let efficiency = compute_efficiency_metrics(env_root).unwrap_or_else(|_| EfficiencyMetrics {
        total_commands: 0,
        unique_commands: 0,
        error_count: 0,
        retry_count: 0,
        help_invocations: 0,
        first_try_success_rate: 0.0,
        iteration_ratio: 0.0,
    });
    let quality = compute_quality_metrics(env_root).unwrap_or_else(|_| QualityMetrics {
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
    let composite_score = compute_composite_score(
        judge_score,
        gates_passed,
        scenario.evaluation.gates.len(),
        &efficiency,
        &quality,
    );

    // Compute counts for final metrics (failures are gracefully handled)
    let note_count = count_notes(env_root).unwrap_or(0);
    let link_count = count_links(env_root).unwrap_or(0);

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

#[allow(dead_code)]
fn get_transcript_summary(env_root: &Path) -> Result<String> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    match std::fs::read_to_string(&transcript_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let summary_len = lines.len().min(100);
            Ok(lines[..summary_len].join("\n"))
        }
        Err(_) => Ok("[No transcript available]".to_string()),
    }
}

#[allow(dead_code)]
fn get_store_export(env_root: &Path) -> Result<String> {
    match run_qipu_json(&["export"], env_root) {
        Ok(json) => Ok(serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())),
        Err(_) => Ok("[No export available]".to_string()),
    }
}

fn get_qipu_path() -> String {
    let paths = [
        "target/release/qipu",
        "target/debug/qipu",
        "../../target/release/qipu",
        "../../target/debug/qipu",
    ];

    for path in paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Fallback
    "target/debug/qipu".to_string()
}

// Helper to run qipu command and parse JSON output
fn run_qipu_json(args: &[&str], env_root: &Path) -> Result<serde_json::Value> {
    let qipu = get_qipu_path();
    // We resolve qipu path relative to current working directory (workspace root)
    // before passing it to Command which will run in env_root.
    let qipu_abs = std::fs::canonicalize(&qipu)
        .with_context(|| format!("Could not find qipu binary at {}", qipu))?;

    let output = Command::new(qipu_abs)
        .args(args)
        .arg("--format")
        .arg("json")
        .current_dir(env_root)
        .output()
        .with_context(|| format!("Failed to execute qipu {:?}", args))?;

    if !output.status.success() {
        // Try to parse error message if possible, or just return stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("qipu command failed: {}", stderr);
    }

    serde_json::from_slice(&output.stdout).context("Failed to parse JSON output")
}

fn count_notes(env_root: &Path) -> Result<usize> {
    let json = run_qipu_json(&["list"], env_root)?;
    if let Some(arr) = json.as_array() {
        Ok(arr.len())
    } else {
        Ok(0)
    }
}

fn count_links(env_root: &Path) -> Result<usize> {
    let json = run_qipu_json(&["export"], env_root)?;

    if let Some(links) = json.get("links") {
        if let Some(arr) = links.as_array() {
            return Ok(arr.len());
        }
    }
    Ok(0)
}

fn search_hit(query: &str, env_root: &Path) -> Result<bool> {
    // Ensure index is up to date (sync might fail if no changes, but usually safe)
    let _ = run_qipu_json(&["sync"], env_root);

    let json = run_qipu_json(&["search", query], env_root)?;
    if let Some(arr) = json.as_array() {
        Ok(!arr.is_empty())
    } else {
        Ok(false)
    }
}

fn note_exists(id: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["show", id], env_root).context("Failed to run qipu show")?;
    // If we get a valid JSON response with an "id" field, the note exists
    Ok(json.get("id").is_some())
}

fn link_exists(from: &str, to: &str, link_type: &str, env_root: &Path) -> Result<bool> {
    let json =
        run_qipu_json(&["link", "list", from], env_root).context("Failed to run qipu link list")?;
    if let Some(arr) = json.as_array() {
        for link in arr {
            let id = link.get("id").and_then(|v| v.as_str());
            let typ = link.get("type").and_then(|v| v.as_str());
            if id == Some(to) && typ == Some(link_type) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn tag_exists(tag: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["list"], env_root).context("Failed to run qipu list")?;
    if let Some(arr) = json.as_array() {
        for note in arr {
            if let Some(tags) = note.get("tags").and_then(|v| v.as_array()) {
                for t in tags {
                    if let Some(tag_str) = t.as_str() {
                        if tag_str == tag {
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

fn content_contains(id: &str, substring: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["show", id], env_root).context("Failed to run qipu show")?;
    let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let body = json.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let content = format!("{}\n{}", title, body);
    Ok(content.contains(substring))
}

fn command_succeeds(command: &str, env_root: &Path) -> Result<bool> {
    let qipu = get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu)
        .with_context(|| format!("Could not find qipu binary at {}", qipu))?;

    // Split the command string into parts (simple shell-like parsing)
    let parts: Vec<&str> = command.split_whitespace().collect();

    let output = Command::new(qipu_abs)
        .args(&parts)
        .current_dir(env_root)
        .output()
        .with_context(|| format!("Failed to execute qipu {}", command))?;

    Ok(output.status.success())
}

fn doctor_passes(env_root: &Path) -> Result<bool> {
    let qipu = get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu)
        .with_context(|| format!("Could not find qipu binary at {}", qipu))?;

    let output = Command::new(qipu_abs)
        .arg("doctor")
        .current_dir(env_root)
        .output()
        .context("Failed to execute qipu doctor")?;

    Ok(output.status.success())
}

fn no_transcript_errors(env_root: &Path) -> Result<bool> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    let content = std::fs::read_to_string(&transcript_path)
        .context("Failed to read transcript file (missing or unreadable)")?;
    let metrics = TranscriptAnalyzer::analyze_with_exit_codes(&content);
    Ok(metrics.error_count == 0)
}

fn compute_efficiency_metrics(env_root: &Path) -> Result<EfficiencyMetrics> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    let content = std::fs::read_to_string(&transcript_path)
        .context("Failed to read transcript file for efficiency metrics")?;
    Ok(TranscriptAnalyzer::analyze_with_exit_codes(&content))
}

fn compute_quality_metrics(env_root: &Path) -> Result<QualityMetrics> {
    let json = run_qipu_json(&["export"], env_root)
        .context("Failed to run qipu export for quality metrics")?;
    let export_json = serde_json::to_string(&json).context("Failed to serialize export JSON")?;
    StoreAnalyzer::analyze(&export_json).context("Failed to analyze store quality")
}

fn compute_composite_score(
    judge_score: Option<f64>,
    gates_passed: usize,
    gates_total: usize,
    efficiency: &EfficiencyMetrics,
    quality: &QualityMetrics,
) -> f64 {
    const JUDGE_WEIGHT: f64 = 0.50;
    const GATES_WEIGHT: f64 = 0.30;
    const EFFICIENCY_WEIGHT: f64 = 0.10;
    const QUALITY_WEIGHT: f64 = 0.10;

    let judge_component = judge_score.unwrap_or(0.0);

    let gates_component = if gates_total > 0 {
        gates_passed as f64 / gates_total as f64
    } else {
        0.0
    };

    let efficiency_component = efficiency.first_try_success_rate;

    let quality_component = if quality.total_notes > 0 {
        let tags_score = quality.avg_tags_per_note.min(3.0) / 3.0;
        let links_score = quality.links_per_note.min(2.0) / 2.0;
        let orphan_penalty =
            (quality.orphan_notes as f64 / quality.total_notes as f64).min(1.0) * 0.3;
        (tags_score + links_score) / 2.0 - orphan_penalty
    } else {
        0.0
    };

    let composite = (JUDGE_WEIGHT * judge_component)
        + (GATES_WEIGHT * gates_component)
        + (EFFICIENCY_WEIGHT * efficiency_component)
        + (QUALITY_WEIGHT * quality_component);

    composite.max(0.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{Evaluation, Gate, Task};
    use tempfile::tempdir;

    fn setup_env() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // Init qipu
        let qipu = get_qipu_path();
        let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");

        let output = Command::new(qipu_abs)
            .arg("init")
            .current_dir(&path)
            .output()
            .expect("failed to run qipu init");

        assert!(output.status.success());

        (dir, path)
    }

    fn create_note_with_stdin(env_root: &Path, content: &str) {
        let qipu = get_qipu_path();
        let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");

        let mut child = Command::new(qipu_abs)
            .arg("capture")
            .current_dir(env_root)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn");

        {
            use std::io::Write;
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(content.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(
            output.status.success(),
            "Capture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_gates() {
        let (_dir, env_root) = setup_env();

        // 1. Empty store - MinNotes 1 should fail
        let scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::MinNotes { count: 1 }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };

        let metrics = evaluate(&scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);
        assert!(!metrics.details[0].passed);

        // 2. Add note
        create_note_with_stdin(&env_root, "This is a test note #test");

        // 3. MinNotes 1 should pass
        let metrics = evaluate(&scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);
        assert!(metrics.details[0].passed);

        // 4. MinNotes 2 should fail
        let scenario_fail_2 = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::MinNotes { count: 2 }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_fail_2, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 5. Search hit
        let scenario_search = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::SearchHit {
                    query: "test".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_search, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        let scenario_search_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::SearchHit {
                    query: "nonexistent".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_search_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 6. NoteExists - get the first note ID from list
        let json = run_qipu_json(&["list"], &env_root).unwrap();
        let first_note_id = json
            .get(0)
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .expect("No notes found");

        // 7. NoteExists should pass with existing ID
        let scenario_note_exists = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::NoteExists {
                    id: first_note_id.to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_note_exists, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 8. NoteExists should fail with non-existent ID
        let scenario_note_exists_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::NoteExists {
                    id: "qp-nonexistent".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_note_exists_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 9. LinkExists - should fail with non-existent link
        let link_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::LinkExists {
                    from: first_note_id.to_string(),
                    to: first_note_id.to_string(),
                    link_type: "related".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&link_scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 10. LinkExists should pass with existing link - first create second note
        create_note_with_stdin(&env_root, "Second note for link test");
        let json = run_qipu_json(&["list"], &env_root).unwrap();
        // Find the note that's different from first_note_id
        let second_note_id = json
            .as_array()
            .and_then(|arr| {
                arr.iter().find_map(|v| {
                    let id = v.get("id").and_then(|v| v.as_str());
                    if id != Some(first_note_id) {
                        id
                    } else {
                        None
                    }
                })
            })
            .expect("Second note not found");

        // Now create a link
        let qipu = get_qipu_path();
        let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");
        let output = Command::new(qipu_abs)
            .args([
                "link",
                "add",
                first_note_id,
                second_note_id,
                "--type",
                "related",
            ])
            .current_dir(&env_root)
            .output()
            .expect("failed to run qipu link add");
        assert!(
            output.status.success(),
            "Link add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Test LinkExists with the new link
        let link_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::LinkExists {
                    from: first_note_id.to_string(),
                    to: second_note_id.to_string(),
                    link_type: "related".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&link_scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 11. TagExists - should fail with non-existent tag
        let tag_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::TagExists {
                    tag: "nonexistent".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&tag_scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 12. TagExists should pass with existing tag - create a note with tag
        let qipu = get_qipu_path();
        let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");
        let output = Command::new(qipu_abs)
            .args(["create", "Important note", "--tag", "important"])
            .current_dir(&env_root)
            .output()
            .expect("failed to run qipu create");
        assert!(
            output.status.success(),
            "Create failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let tag_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::TagExists {
                    tag: "important".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&tag_scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 13. ContentContains - should pass with existing content
        let content_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::ContentContains {
                    id: first_note_id.to_string(),
                    substring: "test note".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&content_scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 14. ContentContains - should fail with non-existent substring
        let content_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::ContentContains {
                    id: first_note_id.to_string(),
                    substring: "nonexistent".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&content_scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 15. CommandSucceeds - should pass with successful command
        let command_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::CommandSucceeds {
                    command: "list".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&command_scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 16. CommandSucceeds - should fail with failing command
        let command_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::CommandSucceeds {
                    command: "nonexistent-command".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&command_scenario_fail, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);
    }

    #[test]
    fn test_compute_composite_score_with_judge() {
        let efficiency = EfficiencyMetrics {
            total_commands: 5,
            unique_commands: 3,
            error_count: 0,
            retry_count: 1,
            help_invocations: 0,
            first_try_success_rate: 0.8,
            iteration_ratio: 1.5,
        };

        let quality = QualityMetrics {
            avg_title_length: 10.0,
            avg_body_length: 50.0,
            avg_tags_per_note: 2.0,
            notes_without_tags: 0,
            links_per_note: 1.0,
            orphan_notes: 0,
            link_type_diversity: 1,
            type_distribution: std::collections::HashMap::new(),
            total_notes: 10,
            total_links: 10,
        };

        let composite = compute_composite_score(Some(0.9), 3, 3, &efficiency, &quality);

        let tags_score = (2.0_f64).min(3.0) / 3.0;
        let links_score = (1.0_f64).min(2.0) / 2.0;
        let orphan_penalty = 0.0;
        let quality_component = (tags_score + links_score) / 2.0 - orphan_penalty;

        let expected = (0.50 * 0.9) + (0.30 * 1.0) + (0.10 * 0.8) + (0.10 * quality_component);
        assert!((composite - expected).abs() < 0.001);
    }

    #[test]
    fn test_compute_composite_score_without_judge() {
        let efficiency = EfficiencyMetrics {
            total_commands: 5,
            unique_commands: 3,
            error_count: 0,
            retry_count: 1,
            help_invocations: 0,
            first_try_success_rate: 0.8,
            iteration_ratio: 1.5,
        };

        let quality = QualityMetrics {
            avg_title_length: 10.0,
            avg_body_length: 50.0,
            avg_tags_per_note: 2.0,
            notes_without_tags: 0,
            links_per_note: 1.0,
            orphan_notes: 0,
            link_type_diversity: 1,
            type_distribution: std::collections::HashMap::new(),
            total_notes: 10,
            total_links: 10,
        };

        let composite = compute_composite_score(None, 3, 3, &efficiency, &quality);

        let tags_score = (2.0_f64).min(3.0) / 3.0;
        let links_score = (1.0_f64).min(2.0) / 2.0;
        let orphan_penalty = 0.0;
        let quality_component = (tags_score + links_score) / 2.0 - orphan_penalty;

        let expected = (0.50 * 0.0) + (0.30 * 1.0) + (0.10 * 0.8) + (0.10 * quality_component);
        assert!((composite - expected).abs() < 0.001);
    }

    #[test]
    fn test_compute_composite_score_empty_store() {
        let efficiency = EfficiencyMetrics {
            total_commands: 0,
            unique_commands: 0,
            error_count: 0,
            retry_count: 0,
            help_invocations: 0,
            first_try_success_rate: 0.0,
            iteration_ratio: 0.0,
        };

        let quality = QualityMetrics {
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
        };

        let composite = compute_composite_score(None, 0, 0, &efficiency, &quality);

        assert_eq!(composite, 0.0);
    }

    #[test]
    fn test_compute_composite_score_clamped() {
        let efficiency = EfficiencyMetrics {
            total_commands: 5,
            unique_commands: 3,
            error_count: 0,
            retry_count: 1,
            help_invocations: 0,
            first_try_success_rate: 1.5,
            iteration_ratio: 1.5,
        };

        let quality = QualityMetrics {
            avg_title_length: 10.0,
            avg_body_length: 50.0,
            avg_tags_per_note: 10.0,
            notes_without_tags: 0,
            links_per_note: 10.0,
            orphan_notes: 0,
            link_type_diversity: 1,
            type_distribution: std::collections::HashMap::new(),
            total_notes: 10,
            total_links: 10,
        };

        let composite = compute_composite_score(Some(1.5), 3, 3, &efficiency, &quality);

        assert!(composite <= 1.0);
        assert!(composite >= 0.0);
    }

    #[test]
    fn test_score_tier_excellent() {
        assert_eq!(ScoreTier::from_score(0.95), ScoreTier::Excellent);
        assert_eq!(ScoreTier::from_score(0.90), ScoreTier::Excellent);
        assert_eq!(ScoreTier::from_score(1.00), ScoreTier::Excellent);
    }

    #[test]
    fn test_score_tier_good() {
        assert_eq!(ScoreTier::from_score(0.85), ScoreTier::Good);
        assert_eq!(ScoreTier::from_score(0.75), ScoreTier::Good);
        assert_eq!(ScoreTier::from_score(0.70), ScoreTier::Good);
    }

    #[test]
    fn test_score_tier_acceptable() {
        assert_eq!(ScoreTier::from_score(0.65), ScoreTier::Acceptable);
        assert_eq!(ScoreTier::from_score(0.55), ScoreTier::Acceptable);
        assert_eq!(ScoreTier::from_score(0.50), ScoreTier::Acceptable);
    }

    #[test]
    fn test_score_tier_poor() {
        assert_eq!(ScoreTier::from_score(0.45), ScoreTier::Poor);
        assert_eq!(ScoreTier::from_score(0.25), ScoreTier::Poor);
        assert_eq!(ScoreTier::from_score(0.00), ScoreTier::Poor);
    }

    #[test]
    fn test_score_tier_boundary_cases() {
        assert_eq!(ScoreTier::from_score(0.8999), ScoreTier::Good);
        assert_eq!(ScoreTier::from_score(0.9000), ScoreTier::Excellent);
        assert_eq!(ScoreTier::from_score(0.6999), ScoreTier::Acceptable);
        assert_eq!(ScoreTier::from_score(0.7000), ScoreTier::Good);
        assert_eq!(ScoreTier::from_score(0.4999), ScoreTier::Poor);
        assert_eq!(ScoreTier::from_score(0.5000), ScoreTier::Acceptable);
    }

    #[test]
    fn test_score_tier_display() {
        assert_eq!(format!("{}", ScoreTier::Excellent), "Excellent");
        assert_eq!(format!("{}", ScoreTier::Good), "Good");
        assert_eq!(format!("{}", ScoreTier::Acceptable), "Acceptable");
        assert_eq!(format!("{}", ScoreTier::Poor), "Poor");
    }

    #[test]
    fn test_doctor_passes_gate() {
        let (_dir, env_root) = setup_env();

        // Create a note so the store is valid
        create_note_with_stdin(&env_root, "Test note for doctor check");

        // DoctorPasses gate should pass with valid store
        let scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::DoctorPasses],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };
        let metrics = evaluate(&scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);
        assert!(metrics.details[0].passed);

        // Break the store by deleting a note file but not from database
        let json = run_qipu_json(&["list"], &env_root).unwrap();
        let first_note_path = json
            .get(0)
            .and_then(|v| v.get("path"))
            .and_then(|v| v.as_str())
            .expect("No path found");

        let note_path = env_root.join(first_note_path);
        std::fs::remove_file(&note_path).expect("Failed to delete note file");

        // DoctorPasses gate should fail with broken store
        let metrics = evaluate(&scenario_pass, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);
        assert!(!metrics.details[0].passed);
    }

    #[test]
    fn test_no_transcript_errors_gate() {
        let (_dir, env_root) = setup_env();

        // Create artifacts directory and transcript
        let artifacts_dir = env_root.join("artifacts");
        std::fs::create_dir_all(&artifacts_dir).unwrap();

        // Test with no errors
        let transcript_no_errors = "qipu create --title 'Test'\nqp-abc123\nqipu list\n...";
        std::fs::write(
            artifacts_dir.join("transcript.raw.txt"),
            transcript_no_errors,
        )
        .unwrap();

        let scenario = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            template_folder: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::NoTranscriptErrors],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
            tags: vec![],
            run: None,
        };

        let metrics = evaluate(&scenario, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 1);
        assert!(metrics.details[0].passed);

        // Test with errors
        let transcript_with_errors = "qipu create --title 'Test'\nError: invalid input\nExit code: 1\nqipu create --title 'Test 2'\nqp-abc123";
        std::fs::write(
            artifacts_dir.join("transcript.raw.txt"),
            transcript_with_errors,
        )
        .unwrap();

        let metrics = evaluate(&scenario, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);
        assert!(!metrics.details[0].passed);

        // Test with missing transcript
        std::fs::remove_file(artifacts_dir.join("transcript.raw.txt")).unwrap();

        let metrics = evaluate(&scenario, &env_root, false).unwrap();
        assert_eq!(metrics.gates_passed, 0);
        assert!(!metrics.details[0].passed);
    }
}
