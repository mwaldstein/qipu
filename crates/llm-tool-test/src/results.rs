use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRecord {
    pub id: String,
    pub scenario_id: String,
    pub scenario_hash: String,
    pub tool: String,
    pub model: String,
    pub qipu_commit: String,
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f64,
    pub cost_usd: f64,
    pub gates_passed: bool,
    pub metrics: EvaluationMetricsRecord,
    pub judge_score: Option<f64>,
    pub outcome: String,
    pub transcript_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetricsRecord {
    pub gates_passed: usize,
    pub gates_total: usize,
    pub note_count: usize,
    pub link_count: usize,
    pub details: Vec<GateResultRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResultRecord {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

pub struct ResultsDB {
    results_path: PathBuf,
}

impl ResultsDB {
    pub fn new(base_dir: &Path) -> Self {
        let results_dir = base_dir.join("results");
        std::fs::create_dir_all(&results_dir).ok();
        Self {
            results_path: results_dir.join("results.jsonl"),
        }
    }

    pub fn append(&self, record: &ResultRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.results_path)
            .context("Failed to open results.jsonl")?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{}", line).context("Failed to write to results.jsonl")?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<ResultRecord>> {
        if !self.results_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.results_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line from results.jsonl")?;
            let record: ResultRecord =
                serde_json::from_str(&line).context("Failed to parse result record")?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn load_by_id(&self, id: &str) -> Result<Option<ResultRecord>> {
        let records = self.load_all()?;
        Ok(records.into_iter().find(|r| r.id == id))
    }

    pub fn load_latest_by_scenario(&self, scenario_id: &str) -> Result<Option<ResultRecord>> {
        let mut records = self.load_all()?;
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(records.into_iter().find(|r| r.scenario_id == scenario_id))
    }

    pub fn load_baseline(&self, scenario_id: &str, tool: &str) -> Result<Option<ResultRecord>> {
        let mut records = self.load_all()?;
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(records
            .into_iter()
            .find(|r| r.scenario_id == scenario_id && r.tool == tool))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub scenario_hash: String,
    pub prompt_hash: String,
    pub tool: String,
    pub qipu_version: String,
}

impl CacheKey {
    pub fn compute(scenario_yaml: &str, prompt: &str, tool: &str, qipu_version: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(scenario_yaml.as_bytes());
        let scenario_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let prompt_hash = format!("{:x}", hasher.finalize());

        Self {
            scenario_hash,
            prompt_hash,
            tool: tool.to_string(),
            qipu_version: qipu_version.to_string(),
        }
    }

    pub fn as_string(&self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.scenario_hash, self.prompt_hash, self.tool, self.qipu_version
        )
    }
}

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(base_dir: &Path) -> Self {
        let cache_dir = base_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    pub fn get(&self, key: &CacheKey) -> Option<ResultRecord> {
        let cache_file = self.cache_dir.join(key.as_string());
        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn put(&self, key: &CacheKey, record: &ResultRecord) -> Result<()> {
        let cache_file = self.cache_dir.join(key.as_string());
        let content = serde_json::to_string_pretty(record)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let path = entry?.path();
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegressionReport {
    pub run_id: String,
    pub baseline_id: String,
    pub score_change_pct: Option<f64>,
    pub cost_change_pct: f64,
    pub warnings: Vec<String>,
    pub alerts: Vec<String>,
}

pub fn compare_runs(current: &ResultRecord, baseline: &ResultRecord) -> RegressionReport {
    let mut warnings = Vec::new();
    let mut alerts = Vec::new();

    let cost_change_pct = if baseline.cost_usd > 0.0 {
        ((current.cost_usd - baseline.cost_usd) / baseline.cost_usd) * 100.0
    } else {
        0.0
    };

    if cost_change_pct > 50.0 {
        warnings.push(format!(
            "Cost increased by {:.1}% ({} -> {})",
            cost_change_pct, baseline.cost_usd, current.cost_usd
        ));
    }

    let score_change_pct = if let (Some(current_score), Some(baseline_score)) =
        (current.judge_score, baseline.judge_score)
    {
        if baseline_score > 0.0 {
            let change = ((current_score - baseline_score) / baseline_score) * 100.0;
            if change < -15.0 {
                warnings.push(format!(
                    "Judge score degraded by {:.1}% ({} -> {})",
                    change, baseline_score, current_score
                ));
            }
            Some(change)
        } else {
            None
        }
    } else {
        None
    };

    if baseline.gates_passed && !current.gates_passed {
        alerts.push("Gate failures that previously passed".to_string());
    }

    RegressionReport {
        run_id: current.id.clone(),
        baseline_id: baseline.id.clone(),
        score_change_pct,
        cost_change_pct,
        warnings,
        alerts,
    }
}

pub fn generate_run_id() -> String {
    let now = Utc::now();
    format!("run-{}", now.format("%Y%m%d-%H%M%S-%f"))
}

pub fn get_qipu_version() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir("..")
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(commit[..8].to_string());
        }
    }

    Ok("unknown".to_string())
}
