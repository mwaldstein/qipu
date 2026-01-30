use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn get_qipu_path() -> String {
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

    "target/debug/qipu".to_string()
}

pub fn run_qipu_json(args: &[&str], env_root: &Path) -> Result<serde_json::Value> {
    let qipu = get_qipu_path();
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("qipu command failed: {}", stderr);
    }

    serde_json::from_slice(&output.stdout).context("Failed to parse JSON output")
}

pub fn count_notes(env_root: &Path) -> Result<usize> {
    let json = run_qipu_json(&["list"], env_root)?;
    if let Some(arr) = json.as_array() {
        Ok(arr.len())
    } else {
        Ok(0)
    }
}

pub fn count_links(env_root: &Path) -> Result<usize> {
    let json = run_qipu_json(&["export"], env_root)?;

    if let Some(links) = json.get("links") {
        if let Some(arr) = links.as_array() {
            return Ok(arr.len());
        }
    }
    Ok(0)
}

pub fn search_hit(query: &str, env_root: &Path) -> Result<bool> {
    let _ = run_qipu_json(&["sync"], env_root);

    let json = run_qipu_json(&["search", query], env_root)?;
    if let Some(arr) = json.as_array() {
        Ok(!arr.is_empty())
    } else {
        Ok(false)
    }
}

pub fn note_exists(id: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["show", id], env_root).context("Failed to run qipu show")?;
    Ok(json.get("id").is_some())
}

pub fn link_exists(from: &str, to: &str, link_type: &str, env_root: &Path) -> Result<bool> {
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

pub fn tag_exists(tag: &str, env_root: &Path) -> Result<bool> {
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

pub fn content_contains(id: &str, substring: &str, env_root: &Path) -> Result<bool> {
    let json = run_qipu_json(&["show", id], env_root).context("Failed to run qipu show")?;
    let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let body = json.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let content = format!("{}\n{}", title, body);
    Ok(content.contains(substring))
}

pub fn command_succeeds(command: &str, env_root: &Path) -> Result<bool> {
    let qipu = get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu)
        .with_context(|| format!("Could not find qipu binary at {}", qipu))?;

    let parts: Vec<&str> = command.split_whitespace().collect();

    let output = Command::new(qipu_abs)
        .args(&parts)
        .current_dir(env_root)
        .output()
        .with_context(|| format!("Failed to execute qipu {}", command))?;

    Ok(output.status.success())
}

pub fn doctor_passes(env_root: &Path) -> Result<bool> {
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

pub fn no_transcript_errors(env_root: &Path) -> Result<bool> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    let content = std::fs::read_to_string(&transcript_path)
        .context("Failed to read transcript file (missing or unreadable)")?;
    let metrics = crate::transcript::TranscriptAnalyzer::analyze_with_exit_codes(&content);
    Ok(metrics.error_count == 0)
}

pub fn compute_efficiency_metrics(env_root: &Path) -> Result<crate::transcript::EfficiencyMetrics> {
    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    let content = std::fs::read_to_string(&transcript_path)
        .context("Failed to read transcript file for efficiency metrics")?;
    Ok(crate::transcript::TranscriptAnalyzer::analyze_with_exit_codes(&content))
}

pub fn compute_quality_metrics(env_root: &Path) -> Result<crate::store_analysis::QualityMetrics> {
    let json = run_qipu_json(&["export"], env_root)
        .context("Failed to run qipu export for quality metrics")?;
    let export_json = serde_json::to_string(&json).context("Failed to serialize export JSON")?;
    crate::store_analysis::StoreAnalyzer::analyze(&export_json)
        .context("Failed to analyze store quality")
}

pub fn compute_composite_score(
    judge_score: Option<f64>,
    gates_passed: usize,
    gates_total: usize,
    efficiency: &crate::transcript::EfficiencyMetrics,
    quality: &crate::store_analysis::QualityMetrics,
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

    composite.clamp(0.0, 1.0)
}

/// Create a note by piping content to `qipu capture` via stdin.
/// This is useful for tests that need to create notes programmatically.
#[allow(dead_code)]
pub fn create_note_with_stdin(env_root: &Path, content: &str) {
    let qipu = get_qipu_path();
    let qipu_abs = std::fs::canonicalize(&qipu).expect("qipu binary not found");

    let mut child = std::process::Command::new(qipu_abs)
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
