use crate::scenario::{Gate, Scenario};
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

    // Always gather stats
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
        };

        if result.passed {
            println!("Gate {} passed: {}", result.gate_type, result.message);
            gates_passed += 1;
        } else {
            println!("Gate {} FAILED: {}", result.gate_type, result.message);
        }
        details.push(result);
    }

    let metrics = EvaluationMetrics {
        gates_passed,
        gates_total: scenario.evaluation.gates.len(),
        note_count,
        link_count,
        details,
    };

    // For now we still bail if any gate failed, to maintain previous behavior in main
    // But maybe main should decide?
    // The plan says "Return metric vector, not just pass/fail".
    // So let's return Ok(metrics) and let caller decide.

    Ok(metrics)
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
            },
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
            },
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
            },
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
            },
        };
        let metrics = evaluate(&scenario_search_fail, &env_root).unwrap();
        assert_eq!(metrics.gates_passed, 0);
    }
}
