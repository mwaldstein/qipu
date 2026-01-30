use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub template_folder: String,
    pub task: Task,
    pub evaluation: Evaluation,
    #[serde(default = "default_tier")]
    pub tier: usize,
    #[serde(default)]
    pub tool_matrix: Option<Vec<ToolConfig>>,
    #[serde(default)]
    pub setup: Option<Setup>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub run: Option<RunConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub max_turns: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub commands: Vec<String>,
}

fn default_tier() -> usize {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool: String,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub gates: Vec<Gate>,
    #[serde(default)]
    pub judge: Option<JudgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    pub enabled: bool,
    pub rubric: String,
    pub pass_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Gate {
    MinNotes {
        count: usize,
    },
    MinLinks {
        count: usize,
    },
    SearchHit {
        query: String,
    },
    NoteExists {
        id: String,
    },
    LinkExists {
        from: String,
        to: String,
        link_type: String,
    },
    TagExists {
        tag: String,
    },
    ContentContains {
        id: String,
        substring: String,
    },
    CommandSucceeds {
        command: String,
    },
    DoctorPasses,
    NoTranscriptErrors,
}
