//! Scenario type definitions for LLM tool testing.
//!
//! This module defines all the data structures used to represent test scenarios,
//! including task definitions, evaluation gates, and tool configurations.

use serde::{Deserialize, Serialize};

/// A test scenario defining a complete LLM tool evaluation case.
///
/// Scenarios are loaded from YAML files and specify:
/// - A task prompt for the LLM tool
/// - Evaluation gates to verify success
/// - Optional setup commands and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Human-readable name for this scenario
    pub name: String,
    /// Detailed description of what this scenario tests
    pub description: String,
    /// Path to the template folder containing initial state
    pub template_folder: String,
    /// The task definition with prompt
    pub task: Task,
    /// Evaluation configuration with gates
    pub evaluation: Evaluation,
    /// Test tier level (default: 0)
    #[serde(default = "default_tier")]
    pub tier: usize,
    /// Optional tool/model matrix configuration
    #[serde(default)]
    pub tool_matrix: Option<Vec<ToolConfig>>,
    /// Optional setup commands to run before the task
    #[serde(default)]
    pub setup: Option<Setup>,
    /// Tags for categorizing scenarios
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional runtime configuration
    #[serde(default)]
    pub run: Option<RunConfig>,
}

/// Runtime configuration for scenario execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Optional timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Optional maximum number of turns/interactions
    #[serde(default)]
    pub max_turns: Option<usize>,
}

/// Setup commands to prepare the test environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    /// Shell commands to execute before running the task
    pub commands: Vec<String>,
}

fn default_tier() -> usize {
    0
}

/// Configuration for a specific tool and its supported models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name (e.g., "opencode", "claude-code")
    pub tool: String,
    /// List of supported model names
    #[serde(default)]
    pub models: Vec<String>,
}

/// The task definition containing the prompt for the LLM tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// The prompt text to send to the LLM tool
    pub prompt: String,
}

/// Evaluation configuration defining how to assess task completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    /// List of evaluation gates that must pass
    pub gates: Vec<Gate>,
    /// Optional judge configuration for LLM-as-judge scoring
    #[serde(default)]
    pub judge: Option<JudgeConfig>,
}

/// Configuration for LLM-as-judge evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    /// Whether judge evaluation is enabled
    pub enabled: bool,
    /// Path to the rubric YAML file
    pub rubric: String,
    /// Minimum score threshold to pass (0.0-1.0)
    pub pass_threshold: f64,
}

/// Evaluation gate types for verifying task completion.
///
/// Each gate represents a specific assertion about the resulting state
/// after the LLM tool has executed the task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Gate {
    /// Asserts minimum number of notes created
    MinNotes {
        /// Required minimum count
        count: usize,
    },
    /// Asserts minimum number of links created
    MinLinks {
        /// Required minimum count
        count: usize,
    },
    /// Asserts a search query returns results
    SearchHit {
        /// Search query string
        query: String,
    },
    /// Asserts a specific note exists by ID
    NoteExists {
        /// Note ID to check for
        id: String,
    },
    /// Asserts a specific link exists
    LinkExists {
        /// Source note ID
        from: String,
        /// Target note ID
        to: String,
        /// Link type (e.g., "related", "derived-from")
        link_type: String,
    },
    /// Asserts a specific tag exists in the store
    TagExists {
        /// Tag name to check for
        tag: String,
    },
    /// Asserts note content contains a substring
    ContentContains {
        /// Note ID to check
        id: String,
        /// Substring to search for
        substring: String,
    },
    /// Asserts a shell command succeeds
    CommandSucceeds {
        /// Shell command to execute
        command: String,
    },
    /// Asserts doctor check passes (no issues found)
    DoctorPasses,
    /// Asserts no errors in the transcript
    NoTranscriptErrors,
}
