use crate::judge::{load_rubric, run_judge, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::store_analysis::{QualityMetrics, StoreAnalyzer};
use crate::transcript::EfficiencyMetrics;
use crate::transcript::TranscriptAnalyzer;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

pub fn evaluate(scenario: &Scenario, env_root: &Path) -> Result<EvaluationMetrics> {
    println!("Evaluating results for scenario: {}", scenario.name);

    let mut details = Vec::new();
    let mut gates_passed = 0;

    let note_count = count_notes(env_root).unwrap_or(0);
    let link_count = count_links(env_root).unwrap_or(0);

    for gate in &scenario.evaluation.gates {
        let result = match gate {
            Gate::MinNotes { count } => {
                let passed = note_count >= *count;
                GateResult {
                    gate_type: "MinNotes".to_string(),
                    passed,
                    message: format!("Expected >= {}, found {}", count, note_count),
                }
            }
            Gate::MinLinks { count } => {
                let passed = link_count >= *count;
                GateResult {
                    gate_type: "MinLinks".to_string(),
                    passed,
                    message: format!("Expected >= {}, found {}", count, link_count),
                }
            }
            Gate::SearchHit { query } => {
                let hit = search_hit(query, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "SearchHit".to_string(),
                    passed: hit,
                    message: format!("Query '{}' found: {}", query, hit),
                }
            }
            Gate::NoteExists { id } => {
                let exists = note_exists(id, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "NoteExists".to_string(),
                    passed: exists,
                    message: format!("Note '{}' exists: {}", id, exists),
                }
            }
            Gate::LinkExists {
                from,
                to,
                link_type,
            } => {
                let exists = link_exists(from, to, link_type, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "LinkExists".to_string(),
                    passed: exists,
                    message: format!(
                        "Link {} --[{}]--> {} exists: {}",
                        from, link_type, to, exists
                    ),
                }
            }
            Gate::TagExists { tag } => {
                let exists = tag_exists(tag, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "TagExists".to_string(),
                    passed: exists,
                    message: format!("Tag '{}' exists: {}", tag, exists),
                }
            }
            Gate::ContentContains { id, substring } => {
                let contains = content_contains(id, substring, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "ContentContains".to_string(),
                    passed: contains,
                    message: format!("Note '{}' contains '{}': {}", id, substring, contains),
                }
            }
            Gate::CommandSucceeds { command } => {
                let succeeds = command_succeeds(command, env_root).unwrap_or(false);
                GateResult {
                    gate_type: "CommandSucceeds".to_string(),
                    passed: succeeds,
                    message: format!("Command '{}' succeeded: {}", command, succeeds),
                }
            }
        };

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
        if judge_config.enabled {
            println!("Running LLM-as-judge evaluation...");
            let model =
                std::env::var("LLM_TOOL_TEST_JUDGE").unwrap_or_else(|_| "gpt-4o-mini".to_string());

            let rubric_path = env_root.join(&judge_config.rubric);
            let rubric = load_rubric(&rubric_path)
                .with_context(|| format!("Failed to load rubric from {}", judge_config.rubric))?;

            let transcript_summary = get_transcript_summary(env_root)?;
            let store_export = get_store_export(env_root)?;

            let runtime = tokio::runtime::Runtime::new()
                .context("Failed to create tokio runtime for judge")?;

            let response = runtime
                .block_on(run_judge(
                    &model,
                    &transcript_summary,
                    &store_export,
                    &scenario.task.prompt,
                    &rubric,
                ))
                .context("LLM judge evaluation failed")?;

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

    let efficiency = compute_efficiency_metrics(env_root)?;
    let quality = compute_quality_metrics(env_root)?;
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
    };

    Ok(metrics)
}

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
    let json = run_qipu_json(&["show", id], env_root);
    match json {
        Ok(value) => {
            // If we get a valid JSON response with an "id" field, the note exists
            if value.get("id").is_some() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

fn link_exists(from: &str, to: &str, link_type: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["link", "list", from], env_root);
    match json {
        Ok(value) => {
            if let Some(arr) = value.as_array() {
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
        Err(_) => Ok(false),
    }
}

fn tag_exists(tag: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["list"], env_root);
    match json {
        Ok(value) => {
            if let Some(arr) = value.as_array() {
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
        Err(_) => Ok(false),
    }
}

fn content_contains(id: &str, substring: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["show", id], env_root);
    match json {
        Ok(value) => {
            let title = value.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let body = value.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let content = format!("{}\n{}", title, body);
            Ok(content.contains(substring))
        }
        Err(_) => Ok(false),
    }
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

fn compute_efficiency_metrics(env_root: &Path) -> Result<EfficiencyMetrics> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    match std::fs::read_to_string(&transcript_path) {
        Ok(content) => Ok(TranscriptAnalyzer::analyze_with_exit_codes(&content)),
        Err(_) => Ok(EfficiencyMetrics {
            total_commands: 0,
            unique_commands: 0,
            error_count: 0,
            retry_count: 0,
            help_invocations: 0,
            first_try_success_rate: 0.0,
            iteration_ratio: 0.0,
        }),
    }
}

fn compute_quality_metrics(env_root: &Path) -> Result<QualityMetrics> {
    match run_qipu_json(&["export"], env_root) {
        Ok(json) => {
            let export_json = serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string());
            match StoreAnalyzer::analyze(&export_json) {
                Ok(metrics) => Ok(metrics),
                Err(_) => Ok(QualityMetrics {
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
                }),
            }
        }
        Err(_) => Ok(QualityMetrics {
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
        }),
    }
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
            fixture: "test".to_string(),
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
        };

        let metrics = evaluate(&scenario_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);
        assert!(!metrics.details[0].passed);

        // 2. Add note
        create_note_with_stdin(&env_root, "This is a test note #test");

        // 3. MinNotes 1 should pass
        let metrics = evaluate(&scenario_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);
        assert!(metrics.details[0].passed);

        // 4. MinNotes 2 should fail
        let scenario_fail_2 = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&scenario_fail_2, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 5. Search hit
        let scenario_search = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&scenario_search, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        let scenario_search_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::SearchHit {
                    query: "missing".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
        };
        let metrics = evaluate(&scenario_search_fail, &env_root).unwrap();
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
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&scenario_note_exists, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 8. NoteExists should fail with non-existent ID
        let scenario_note_exists_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&scenario_note_exists_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 9. LinkExists - should fail with non-existent link
        let link_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::LinkExists {
                    from: first_note_id.to_string(),
                    to: "qp-nonexistent".to_string(),
                    link_type: "related".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
        };
        let metrics = evaluate(&link_scenario_fail, &env_root).unwrap();
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
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&link_scenario_pass, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 11. TagExists - should fail with non-existent tag
        let tag_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&tag_scenario_fail, &env_root).unwrap();
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
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&tag_scenario_pass, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 13. ContentContains - should pass with existing content
        let content_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::ContentContains {
                    id: first_note_id.to_string(),
                    substring: "test".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
        };
        let metrics = evaluate(&content_scenario_pass, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 14. ContentContains - should fail with non-existent substring
        let content_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::ContentContains {
                    id: first_note_id.to_string(),
                    substring: "nonexistent text".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
        };
        let metrics = evaluate(&content_scenario_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);

        // 15. CommandSucceeds - should pass with successful command
        let command_scenario_pass = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
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
        };
        let metrics = evaluate(&command_scenario_pass, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 1);

        // 16. CommandSucceeds - should fail with failing command
        let command_scenario_fail = Scenario {
            name: "test".to_string(),
            description: "test".to_string(),
            fixture: "test".to_string(),
            task: Task {
                prompt: "test".to_string(),
            },
            evaluation: Evaluation {
                gates: vec![Gate::CommandSucceeds {
                    command: "show qp-nonexistent".to_string(),
                }],
                judge: None,
            },
            tier: 0,
            tool_matrix: None,
            setup: None,
        };
        let metrics = evaluate(&command_scenario_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);
    }
}
