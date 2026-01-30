//! Type definitions for test results.
//!
//! This module defines all the data structures used to represent
//! test results, metrics, and cache keys.

pub mod cache_key;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use cache_key::CacheKey;

/// A complete test run result record.
///
/// Contains all metadata and metrics for a single scenario execution,
/// including timing, cost, gate results, and quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRecord {
    /// Unique run identifier
    pub id: String,
    /// Scenario identifier (filename without extension)
    pub scenario_id: String,
    /// Hash of the scenario YAML content
    pub scenario_hash: String,
    /// Tool name (e.g., "opencode", "claude-code")
    pub tool: String,
    /// Model name used for this run
    pub model: String,
    /// Qipu git commit hash (short)
    pub qipu_commit: String,
    /// Timestamp when the run completed
    pub timestamp: DateTime<Utc>,
    /// Total duration in seconds
    pub duration_secs: f64,
    /// Estimated cost in USD
    pub cost_usd: f64,
    /// Whether all gates passed
    pub gates_passed: bool,
    /// Detailed evaluation metrics
    pub metrics: EvaluationMetricsRecord,
    /// Optional LLM-as-judge score (0.0-1.0)
    pub judge_score: Option<f64>,
    /// Final outcome ("PASS", "FAIL", "ERROR")
    pub outcome: String,
    /// Path to the saved transcript file
    pub transcript_path: String,
    /// Optional cache key for this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

/// Evaluation metrics for a test run.
///
/// Aggregates gate results, efficiency metrics, quality metrics,
/// and a composite quality score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetricsRecord {
    /// Number of gates that passed
    pub gates_passed: usize,
    /// Total number of gates evaluated
    pub gates_total: usize,
    /// Total notes created in the store
    pub note_count: usize,
    /// Total links created in the store
    pub link_count: usize,
    /// Detailed results for each gate
    pub details: Vec<GateResultRecord>,
    /// Efficiency metrics
    pub efficiency: EfficiencyMetricsRecord,
    /// Quality metrics
    pub quality: QualityMetricsRecord,
    /// Composite quality score (0.0-1.0)
    pub composite_score: f64,
}

/// Efficiency metrics measuring tool interaction patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetricsRecord {
    /// Total number of commands executed
    pub total_commands: usize,
    /// Number of unique commands executed
    pub unique_commands: usize,
    /// Number of commands that resulted in errors
    pub error_count: usize,
    /// Number of command retries
    pub retry_count: usize,
    /// Number of help invocations
    pub help_invocations: usize,
    /// Rate of commands succeeding on first attempt (0.0-1.0)
    pub first_try_success_rate: f64,
    /// Ratio of total commands to unique commands
    pub iteration_ratio: f64,
}

/// Quality metrics measuring note and link characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsRecord {
    /// Average note title length in characters
    pub avg_title_length: f64,
    /// Average note body length in characters
    pub avg_body_length: f64,
    /// Average number of tags per note
    pub avg_tags_per_note: f64,
    /// Number of notes without any tags
    pub notes_without_tags: usize,
    /// Average links per note
    pub links_per_note: f64,
    /// Number of notes with no incoming or outgoing links
    pub orphan_notes: usize,
    /// Number of distinct link types used
    pub link_type_diversity: usize,
    /// Distribution of note types
    pub type_distribution: HashMap<String, usize>,
    /// Total number of notes
    pub total_notes: usize,
    /// Total number of links
    pub total_links: usize,
}

/// Result of evaluating a single gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResultRecord {
    /// Type of gate evaluated
    pub gate_type: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Human-readable message about the result
    pub message: String,
}
